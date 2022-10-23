package it.delonghi.ecam;

import android.app.Activity;
import android.bluetooth.BluetoothDevice;
import android.bluetooth.BluetoothGatt;
import android.bluetooth.BluetoothGattCallback;
import android.bluetooth.BluetoothGattCharacteristic;
import android.bluetooth.BluetoothGattDescriptor;
import android.bluetooth.BluetoothGattService;
import android.content.Context;
import android.os.AsyncTask;
import android.os.Handler;
import android.util.Log;
import com.google.android.exoplayer2.C;
import com.google.android.exoplayer2.extractor.ts.PsExtractor;
import com.google.common.base.Ascii;
import it.delonghi.Constants;
import it.delonghi.DeLonghiManager;
import it.delonghi.bluetooth.BleManager;
import it.delonghi.bluetooth.BleUtils;
import it.delonghi.bluetooth.itf.CustomLeScanCallback;
import it.delonghi.ecam.itf.EcamUpdatesReceived;
import it.delonghi.ecam.model.EcamRequest;
import it.delonghi.ecam.model.MonitorData;
import it.delonghi.ecam.model.Parameter;
import it.delonghi.ecam.model.RecipeData;
import it.delonghi.ecam.model.enums.BeverageTasteType;
import it.delonghi.ecam.model.enums.BeverageTasteValue;
import it.delonghi.ecam.model.enums.OperationTriggerId;
import it.delonghi.utils.DLog;
import it.delonghi.utils.Utils;
import it.delonghi.utils.comparators.RequestPriorityComparator;
import java.lang.ref.WeakReference;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.PriorityQueue;
import java.util.UUID;
import org.objectweb.asm.Opcodes;

/* loaded from: classes2.dex */
public class EcamManager {
    private static final int ANSWER_ID_BYTE_IDX = 2;
    public static final byte BEVERAGE_DISPENSING_ANSWER_ID = -126;
    private static final byte BUSY_MAIN_BOARD_ANSWER = -31;
    private static final int BUSY_RETRY_DELAY = 100;
    private static final byte CHECKSUM_ANSWER_ID = -93;
    private static final byte DATA_0_ANSWER_ID = 96;
    private static final byte DATA_1_ANSWER_ID = 112;
    public static final byte DATA_2_ANSWER_ID = 117;
    private static final int MAX_BUSY_RETRIES = 10;
    public static final int MAX_CONNECTION_RETRIES = 3;
    private static final int MAX_MISMATCH_RETRIES = 5;
    private static final int MAX_REQUEST_RETRIES = 5;
    private static final int MISMATCH_RETRY_DELAY = 100;
    public static final byte PARAMETER_READ_ANSWER_ID = -107;
    public static final byte PARAMETER_READ_EXT_ANSWER_ID = -95;
    public static final byte PARAMETER_WRITE_ANSWER_ID = -112;
    public static final byte PIN_ACTIVATION_ANSWER_ID = -80;
    public static final byte PIN_SET_ANSWER_ID = -79;
    private static final int PRIORITY_HIGH = 1;
    public static final int PRIORITY_LOW = 3;
    private static final int PRIORITY_NORMAL = 2;
    private static final byte PROFILES_NAME_READ_ANSWER_ID = -92;
    public static final byte PROFILES_NAME_WRITE_ANSWER_ID = -91;
    public static final byte PROFILE_SELECTION_ANSWER_ID = -87;
    private static final byte RECIPES_NAME_READ_ANSWER_ID = -86;
    public static final byte RECIPES_NAME_WRITE_ANSWER_ID = -85;
    public static final byte RECIPES_PRIORITY_READ_ANSWER_ID = -88;
    public static final byte RECIPES_QTY_READ_ANSWER_ID = -90;
    public static final byte SET_TIME_ANSWER = -30;
    private static final int TIMEOUT = 3000;
    private byte[][] chunk;
    private EcamRequest currentRequest;
    private BleManager mBleManager;
    private String mConnectedEcamMachineAddress;
    private BluetoothGattCharacteristic mEcamCharacteristic;
    private ArrayList<String> mEcamMachinesAddresses;
    private Handler mHandler;
    private final PriorityQueue<EcamRequest> mRequestQueue;
    private EcamUpdatesReceived mUpdatesListener;
    private byte[] response;
    private static final String TAG = EcamManager.class.getName();
    private static final UUID[] mEcamServiceUUIDs = {UUID.fromString(Constants.TRANSFER_SERVICE_UUID)};
    private static final UUID mEcamCharacteristicUUID = UUID.fromString(Constants.TRANSFER_CHARACTERISTIC_UUID);
    private int nextChunkIdx = 0;
    private int requestTimeoutRetries = 0;
    private int requestBusyRetries = 0;
    private int responseMismatchRetries = 0;
    private Runnable mTimeout = new Runnable() { // from class: it.delonghi.ecam.EcamManager.1
        @Override // java.lang.Runnable
        public void run() {
            DLog.d(EcamManager.TAG, "Request timeout!");
            EcamManager.this.response = null;
            synchronized (EcamManager.this.mRequestQueue) {
                if (EcamManager.this.currentRequest != null) {
                    if (EcamManager.this.requestTimeoutRetries >= 5) {
                        EcamManager.this.requestTimeoutRetries = 0;
                        EcamManager.this.mUpdatesListener.onRequestTimeout(EcamManager.this.currentRequest.getRequest()[2]);
                    } else {
                        EcamManager.this.currentRequest.setPriority(1);
                        EcamManager.this.mRequestQueue.add(EcamManager.this.currentRequest);
                        EcamManager.access$408(EcamManager.this);
                    }
                    EcamManager.this.currentRequest = null;
                    EcamManager.this.mRequestQueue.notify();
                }
            }
        }
    };
    private Runnable mBusy = new Runnable() { // from class: it.delonghi.ecam.EcamManager.2
        @Override // java.lang.Runnable
        public void run() {
            DLog.d(EcamManager.TAG, "Busy retry");
            EcamManager.this.response = null;
            synchronized (EcamManager.this.mRequestQueue) {
                if (EcamManager.this.currentRequest != null) {
                    if (EcamManager.this.requestBusyRetries >= 10) {
                        EcamManager.this.requestBusyRetries = 0;
                        EcamManager.this.mUpdatesListener.onRequestTimeout(EcamManager.this.currentRequest.getRequest()[2]);
                    } else {
                        EcamManager.this.currentRequest.setPriority(1);
                        EcamManager.this.mRequestQueue.add(EcamManager.this.currentRequest);
                        EcamManager.access$608(EcamManager.this);
                    }
                    EcamManager.this.currentRequest = null;
                    EcamManager.this.mRequestQueue.notify();
                }
            }
        }
    };
    private Runnable mResponseMismatch = new Runnable() { // from class: it.delonghi.ecam.EcamManager.3
        @Override // java.lang.Runnable
        public void run() {
            DLog.d(EcamManager.TAG, "Answer mismatch retry");
            EcamManager.this.response = null;
            synchronized (EcamManager.this.mRequestQueue) {
                if (EcamManager.this.currentRequest != null) {
                    if (EcamManager.this.responseMismatchRetries >= 5) {
                        EcamManager.this.responseMismatchRetries = 0;
                        EcamManager.this.mUpdatesListener.onRequestTimeout(EcamManager.this.currentRequest.getRequest()[2]);
                    } else {
                        EcamManager.this.currentRequest.setPriority(1);
                        EcamManager.this.mRequestQueue.add(EcamManager.this.currentRequest);
                        EcamManager.access$708(EcamManager.this);
                    }
                    EcamManager.this.currentRequest = null;
                    EcamManager.this.mRequestQueue.notify();
                }
            }
        }
    };
    private EcamScanCallback mEcamScanCallback = new EcamScanCallback(this);
    private EcamGattCallback mEcamGattCallback = new EcamGattCallback(this);

    static /* synthetic */ int access$408(EcamManager ecamManager) {
        int i = ecamManager.requestTimeoutRetries;
        ecamManager.requestTimeoutRetries = i + 1;
        return i;
    }

    static /* synthetic */ int access$608(EcamManager ecamManager) {
        int i = ecamManager.requestBusyRetries;
        ecamManager.requestBusyRetries = i + 1;
        return i;
    }

    static /* synthetic */ int access$708(EcamManager ecamManager) {
        int i = ecamManager.responseMismatchRetries;
        ecamManager.responseMismatchRetries = i + 1;
        return i;
    }

    static /* synthetic */ int access$808(EcamManager ecamManager) {
        int i = ecamManager.nextChunkIdx;
        ecamManager.nextChunkIdx = i + 1;
        return i;
    }

    public EcamManager(Context context, EcamUpdatesReceived ecamUpdatesReceived) {
        this.mBleManager = new BleManager(context);
        this.mUpdatesListener = ecamUpdatesReceived;
        this.mBleManager.registerBleScanListener(this.mEcamScanCallback);
        this.mBleManager.registerGattListener(this.mEcamGattCallback);
        this.mHandler = new Handler();
        this.mRequestQueue = new PriorityQueue<>(20, new RequestPriorityComparator());
        new Thread(new Runnable() { // from class: it.delonghi.ecam.EcamManager.4
            @Override // java.lang.Runnable
            public void run() {
                while (true) {
                    byte[] bArr = null;
                    synchronized (EcamManager.this.mRequestQueue) {
                        try {
                            EcamManager.this.mRequestQueue.wait();
                        } catch (InterruptedException e) {
                            String str = EcamManager.TAG;
                            DLog.e(str, "InterruptedException!" + e.getMessage());
                        }
                        if (EcamManager.this.currentRequest == null && EcamManager.this.mRequestQueue.size() > 0) {
                            EcamManager.this.currentRequest = (EcamRequest) EcamManager.this.mRequestQueue.poll();
                            bArr = EcamManager.this.currentRequest.getRequest();
                        }
                        if (bArr != null) {
                            if (bArr.length > BleManager.BLE_PACKET_MAX_SIZE) {
                                EcamManager.this.nextChunkIdx = 0;
                                EcamManager.this.chunk = Utils.divideArray(bArr, BleManager.BLE_PACKET_MAX_SIZE);
                                EcamManager.this.writeBytes(EcamManager.this.chunk[EcamManager.this.nextChunkIdx]);
                            } else if (EcamManager.this.writeBytes(bArr)) {
                                EcamManager.this.mHandler.postDelayed(EcamManager.this.mTimeout, 3000L);
                            } else {
                                EcamManager.this.mHandler.post(EcamManager.this.mTimeout);
                            }
                        }
                    }
                }
            }
        }).start();
    }

    /* JADX INFO: Access modifiers changed from: private */
    public boolean writeBytes(byte[] bArr) {
        BluetoothGattCharacteristic bluetoothGattCharacteristic = this.mEcamCharacteristic;
        if (bluetoothGattCharacteristic == null || this.mConnectedEcamMachineAddress == null) {
            return false;
        }
        bluetoothGattCharacteristic.setValue(bArr);
        this.mEcamCharacteristic.setWriteType(2);
        boolean writeCharacteristic = this.mBleManager.writeCharacteristic(this.mConnectedEcamMachineAddress, this.mEcamCharacteristic);
        if (writeCharacteristic) {
            String str = TAG;
            DLog.test(str, "Wrote packet:" + Utils.byteArrayToHex(bArr));
        }
        return writeCharacteristic;
    }

    private void enqueueRequest(EcamRequest ecamRequest) {
        new EnqueueRequestTask(this).execute(ecamRequest);
    }

    private byte checksum(byte[] bArr) {
        int length = bArr.length - 1;
        byte b = 85;
        for (int i = 0; i < length; i++) {
            b = (byte) (b ^ bArr[i]);
        }
        return b;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public void addEcamMachine(String str, String str2) {
        if (this.mEcamMachinesAddresses == null) {
            this.mEcamMachinesAddresses = new ArrayList<>();
        }
        EcamUpdatesReceived ecamUpdatesReceived = this.mUpdatesListener;
        if (ecamUpdatesReceived != null) {
            ecamUpdatesReceived.onMachineFound(str, str2);
        }
        this.mEcamMachinesAddresses.add(str);
    }

    /* JADX INFO: Access modifiers changed from: private */
    public void scanFinished() {
        EcamUpdatesReceived ecamUpdatesReceived = this.mUpdatesListener;
        if (ecamUpdatesReceived != null) {
            ecamUpdatesReceived.scanBlufi();
        }
    }

    /* JADX INFO: Access modifiers changed from: private */
    public void updateReceived(byte[] bArr) {
        new UpdateReceivedTask(this).execute(bArr);
    }

    /* JADX WARN: Can't fix incorrect switch cases order, some code will duplicate */
    private void decodeResponse(byte[] bArr) {
        byte b = bArr[2];
        if (b == -126) {
            byte b2 = bArr[4];
            DLog.d(TAG, "BEVERAGE_DISPENSING_ANSWER_ID");
            return;
        }
        if (b == -112) {
            this.mUpdatesListener.onParameterWritten(Utils.twoBytesToShort(bArr[4], bArr[5]), bArr[6] == 0);
        } else if (b == -107 || b == -95) {
            int i = ((bArr[1] & 255) - 5) / 4;
            ArrayList<Parameter> arrayList = new ArrayList<>(i);
            int twoBytesToShort = Utils.twoBytesToShort(bArr[4], bArr[5]);
            for (int i2 = 0; i2 < i; i2++) {
                byte[] bArr2 = new byte[4];
                System.arraycopy(bArr, (i2 * 4) + 6, bArr2, 0, 4);
                arrayList.add(new Parameter(twoBytesToShort, bArr2));
                twoBytesToShort++;
            }
            this.mUpdatesListener.onParametersReceived(arrayList);
        } else if (b == 96) {
            this.mUpdatesListener.onMonitorDataReceived(new MonitorData(0, bArr));
        } else if (b == 112) {
            this.mUpdatesListener.onMonitorDataReceived(new MonitorData(1, bArr));
        } else if (b == 117) {
            this.mUpdatesListener.onMonitorDataReceived(new MonitorData(2, bArr));
        } else if (b == -31) {
            this.mUpdatesListener.onRequestTimeout(b);
        } else if (b != -30) {
            switch (b) {
                case -93:
                    short twoBytesToShort2 = Utils.twoBytesToShort(bArr[18], bArr[19]);
                    short twoBytesToShort3 = Utils.twoBytesToShort(bArr[16], bArr[17]);
                    short[] sArr = new short[6];
                    for (int i3 = 0; i3 < 12; i3 += 2) {
                        sArr[i3 / 2] = Utils.twoBytesToShort(bArr[i3 + 4], bArr[i3 + 5]);
                    }
                    this.mUpdatesListener.onChecksumsReceived(twoBytesToShort2, twoBytesToShort3, sArr);
                    return;
                case -92:
                    break;
                case -91:
                    this.mUpdatesListener.onProfilesNamesWritten(bArr[4] == 0);
                    return;
                case -90:
                    byte b3 = bArr[1];
                    this.mUpdatesListener.onRecipesQuantityReceived(Utils.getUnsignedIntFromByte(bArr[4]), getPacketForRecipeData(bArr));
                    return;
                default:
                    switch (b) {
                        case -88:
                            int byteToInt = Utils.byteToInt(bArr[4]);
                            int[] iArr = new int[24];
                            for (int i4 = 0; i4 < 24; i4++) {
                                iArr[i4] = Utils.byteToInt(bArr[i4 + 5]);
                            }
                            this.mUpdatesListener.onPrioritiesReceived(byteToInt, iArr);
                            return;
                        case -87:
                            this.mUpdatesListener.onProfileSelectionAnswer(bArr[4], bArr[5] == 0);
                            return;
                        case -86:
                            break;
                        case -85:
                            this.mUpdatesListener.onRecipesNamesWritten(bArr[4] == 0);
                            return;
                        default:
                            return;
                    }
            }
            int byteToInt2 = (Utils.byteToInt(bArr[1]) - 4) / 21;
            ArrayList<String> arrayList2 = new ArrayList<>(byteToInt2);
            ArrayList<Integer> arrayList3 = new ArrayList<>(byteToInt2);
            for (int i5 = 0; i5 < byteToInt2; i5++) {
                byte[] bArr3 = new byte[20];
                int i6 = i5 * 21;
                System.arraycopy(bArr, i6 + 4, bArr3, 0, 20);
                String str = null;
                if (!Utils.isByteArrayAllZeros(bArr3)) {
                    str = Utils.byteArrayToString(Utils.trim(bArr3), C.UTF16_NAME);
                }
                Integer valueOf = Integer.valueOf(Utils.byteToInt(bArr[i6 + 24]));
                arrayList2.add(str);
                arrayList3.add(valueOf);
            }
            if (b == -92) {
                this.mUpdatesListener.onProfilesNamesReceived(arrayList2, arrayList3);
            } else {
                this.mUpdatesListener.onRecipesNamesReceived(arrayList2, arrayList3);
            }
        } else {
            this.mUpdatesListener.onTimeSet(bArr[4] == 0);
        }
    }

    public static ArrayList<RecipeData> getPacketForRecipeData(byte[] bArr) {
        int i = ((bArr[1] & 255) - 4) / 5;
        ArrayList<RecipeData> arrayList = new ArrayList<>(24);
        int i2 = 0;
        while (i2 < i) {
            int i3 = i2 * 5;
            short twoBytesToShort = Utils.twoBytesToShort(bArr[i3 + 5], bArr[i3 + 6]);
            short twoBytesToShort2 = Utils.twoBytesToShort(bArr[i3 + 7], bArr[i3 + 8]);
            int i4 = i3 + 9;
            int i5 = bArr[i4] & 240;
            boolean z = (bArr[i4] & 4) != 0;
            BeverageTasteValue fromValue = BeverageTasteValue.fromValue(Integer.valueOf(i5));
            i2++;
            RecipeData recipeData = new RecipeData(i2);
            recipeData.setCoffeeQty(twoBytesToShort);
            recipeData.setMilkQty(twoBytesToShort2);
            recipeData.setTasteValue(fromValue);
            recipeData.setInversion(z);
            arrayList.add(recipeData);
        }
        return arrayList;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public boolean isResponseComplete() {
        byte[] bArr = this.response;
        return bArr != null && bArr.length >= 2 && Utils.byteToInt(bArr[1]) == this.response.length - 1;
    }

    public BluetoothDevice getEcamDevice(String str) {
        return this.mBleManager.getBluetoothDevice(str);
    }

    public void startEcamScan() {
        DLog.d(TAG, "startEcamScan");
        this.mBleManager.clearScannedDevices();
        this.mBleManager.scanLeDevice(true, this.mEcamScanCallback);
    }

    public void stopEcamScan() {
        this.mBleManager.scanLeDevice(false, mEcamServiceUUIDs, this.mEcamScanCallback);
    }

    public boolean isScanning() {
        return this.mBleManager.isScanning();
    }

    public boolean isManualDisconnect() {
        return this.mBleManager.ismManualDisconnect();
    }

    public boolean isBleActive() {
        return this.mBleManager.isBtEnabled();
    }

    public String getConnectedEcamMachineAddress() {
        return this.mConnectedEcamMachineAddress;
    }

    public boolean connectToEcamMachine(String str) {
        ArrayList<String> arrayList = this.mEcamMachinesAddresses;
        if (arrayList != null && !arrayList.contains(str)) {
            DLog.w(TAG, "Address not found in scanned Ecam Machines list.");
            return false;
        }
        if (this.mConnectedEcamMachineAddress != null) {
            disconnectFromCurrentEcamMachine();
        }
        return this.mBleManager.connectToDevice(str, false, this.mEcamGattCallback);
    }

    public void disconnectFromCurrentEcamMachine() {
        String str = this.mConnectedEcamMachineAddress;
        if (str == null) {
            DLog.w(TAG, "No machines connected. by disconnectFromCurrentEcamMachine");
            return;
        }
        BluetoothGattCharacteristic bluetoothGattCharacteristic = this.mEcamCharacteristic;
        if (bluetoothGattCharacteristic != null) {
            this.mBleManager.setCharacteristicNotification(str, bluetoothGattCharacteristic, false);
            this.mEcamCharacteristic = null;
        }
        this.response = null;
        this.mBleManager.disconnectFromDevice(this.mConnectedEcamMachineAddress);
    }

    public void silentDisconnectFromCurrentEcamMachine() {
        String str = this.mConnectedEcamMachineAddress;
        if (str == null) {
            DLog.w(TAG, "No machines connected. by silentDisconnectFromCurrentEcamMachine");
            return;
        }
        BluetoothGattCharacteristic bluetoothGattCharacteristic = this.mEcamCharacteristic;
        if (bluetoothGattCharacteristic != null) {
            this.mBleManager.setCharacteristicNotification(str, bluetoothGattCharacteristic, false);
            this.mEcamCharacteristic = null;
        }
        this.response = null;
        this.mBleManager.silentDisconnectFromDevice(this.mConnectedEcamMachineAddress);
    }

    public void getMonitorMode(int i) {
        byte[] bArr = new byte[5];
        bArr[0] = Ascii.CR;
        bArr[1] = 4;
        if (i == 0) {
            bArr[2] = DATA_0_ANSWER_ID;
        } else if (i == 1) {
            bArr[2] = DATA_1_ANSWER_ID;
        } else if (i == 2) {
            bArr[2] = 117;
        }
        bArr[3] = Ascii.SI;
        bArr[4] = checksum(bArr);
        enqueueRequest(new EcamRequest(2, bArr));
    }

    public void dispenseMacchiatoOld() {
        byte[] bArr = {10, 6, BEVERAGE_DISPENSING_ANSWER_ID, Ascii.SI, 1, 0, 55};
        bArr[6] = checksum(bArr);
        enqueueRequest(new EcamRequest(2, bArr));
    }

    public void dispenseBeverage(int i, OperationTriggerId operationTriggerId, int i2, int i3, BeverageTasteValue beverageTasteValue, BeverageTasteType beverageTasteType) {
        byte[] bArr = {Ascii.CR, Ascii.VT, BEVERAGE_DISPENSING_ANSWER_ID, -16, (byte) i, operationTriggerId.getValue(), (byte) (i2 >> 8), (byte) i2, (byte) (i3 >> 8), (byte) i3, (byte) (beverageTasteType.getValue() ^ beverageTasteValue.getValue()), checksum(bArr)};
        enqueueRequest(new EcamRequest(2, bArr));
    }

    public void getParameters(int i, int i2) {
        getParameters(i, i2, 2);
    }

    public void getStatisticalParameters(int i, int i2) {
        getParameters(i, i2, 2);
    }

    private void getParameters(int i, int i2, int i3) {
        if (i2 > 10) {
            i2 = 10;
        }
        byte[] bArr = new byte[8];
        byte b = (byte) (i >> 8);
        byte b2 = (byte) i;
        bArr[0] = Ascii.CR;
        bArr[1] = 7;
        bArr[2] = (byte) (i2 > 4 ? Opcodes.IF_ICMPLT : Opcodes.FCMPL);
        bArr[3] = (byte) (i < 1000 ? 15 : PsExtractor.VIDEO_STREAM_MASK);
        bArr[4] = b;
        bArr[5] = b2;
        bArr[6] = (byte) i2;
        bArr[7] = checksum(bArr);
        enqueueRequest(new EcamRequest(i3, bArr));
    }

    public void setParameter(int i, int i2) {
        byte[] bArr = new byte[11];
        byte b = (byte) (i >> 8);
        byte b2 = (byte) i;
        bArr[0] = Ascii.CR;
        bArr[1] = 10;
        bArr[2] = -112;
        bArr[3] = (byte) (i < 1000 ? 15 : PsExtractor.VIDEO_STREAM_MASK);
        bArr[4] = b;
        bArr[5] = b2;
        bArr[6] = (byte) (i2 >> 24);
        bArr[7] = (byte) (i2 >> 16);
        bArr[8] = (byte) (i2 >> 8);
        bArr[9] = (byte) i2;
        bArr[10] = checksum(bArr);
        enqueueRequest(new EcamRequest(2, bArr));
    }

    public void checksumVerification() {
        byte[] bArr = {Ascii.CR, 4, CHECKSUM_ANSWER_ID, -16, checksum(bArr)};
        enqueueRequest(new EcamRequest(2, bArr));
    }

    public void getRecipesName(int i, int i2) {
        byte[] bArr = {Ascii.CR, 6, -86, -16, (byte) i, (byte) i2, checksum(bArr)};
        enqueueRequest(new EcamRequest(2, bArr));
    }

    public void getProfilesNames(int i, int i2) {
        byte[] bArr = {Ascii.CR, 6, PROFILES_NAME_READ_ANSWER_ID, -16, (byte) i, (byte) i2, checksum(bArr)};
        enqueueRequest(new EcamRequest(2, bArr));
    }

    public void setRecipesName(int i, int i2, String[] strArr, int[] iArr) {
        int length = (strArr.length * 21) + 7;
        byte[] bArr = new byte[length];
        bArr[0] = Ascii.CR;
        int i3 = length - 1;
        bArr[1] = (byte) i3;
        bArr[2] = -85;
        bArr[3] = -16;
        bArr[4] = (byte) i;
        bArr[5] = (byte) i2;
        int i4 = 6;
        for (int i5 = 0; i5 < strArr.length; i5++) {
            byte[] stringToByteArray = Utils.stringToByteArray(strArr[i5]);
            int i6 = 0;
            while (i6 < stringToByteArray.length) {
                bArr[i4] = i6 < 20 ? stringToByteArray[i6] : (byte) 0;
                i4++;
                i6++;
            }
            bArr[i4] = (byte) iArr[i5];
            i4++;
        }
        bArr[i3] = checksum(bArr);
        enqueueRequest(new EcamRequest(2, bArr));
    }

    public void setProfilesNames(int i, int i2, String[] strArr, int[] iArr) {
        int length = (strArr.length * 21) + 7;
        byte[] bArr = new byte[length];
        bArr[0] = Ascii.CR;
        int i3 = length - 1;
        bArr[1] = (byte) i3;
        bArr[2] = PROFILES_NAME_WRITE_ANSWER_ID;
        bArr[3] = -16;
        bArr[4] = (byte) i;
        bArr[5] = (byte) i2;
        int i4 = 6;
        for (int i5 = 0; i5 < strArr.length; i5++) {
            byte[] stringToByteArray = Utils.stringToByteArray(strArr[i5]);
            int i6 = 0;
            while (i6 < stringToByteArray.length) {
                bArr[i4] = i6 < 20 ? stringToByteArray[i6] : (byte) 0;
                i4++;
                i6++;
            }
            bArr[i4] = (byte) iArr[i5];
            i4++;
        }
        bArr[i3] = checksum(bArr);
        enqueueRequest(new EcamRequest(2, bArr));
    }

    public void getRecipesQty(int i, int i2, int i3) {
        byte[] bArr = {Ascii.CR, 7, -90, -16, (byte) i, (byte) i2, (byte) i3, checksum(bArr)};
        enqueueRequest(new EcamRequest(2, bArr));
    }

    public void getRecipesPriority(int i) {
        byte[] bArr = {Ascii.CR, 5, -88, -16, (byte) i, checksum(bArr)};
        enqueueRequest(new EcamRequest(1, bArr));
    }

    public void profileSelection(int i) {
        byte[] bArr = {Ascii.CR, 5, PROFILE_SELECTION_ANSWER_ID, -16, (byte) i, checksum(bArr)};
        enqueueRequest(new EcamRequest(2, bArr));
    }

    public void turnOnMode() {
        byte[] bArr = {Ascii.CR, Ascii.VT, BEVERAGE_DISPENSING_ANSWER_ID, Ascii.SI, 40, 1, 0, 0, 0, 0, 0, checksum(bArr)};
        enqueueRequest(new EcamRequest(2, bArr));
    }

    public void setPinActivation(boolean z) {
        byte[] bArr = {Ascii.CR, 5, PIN_ACTIVATION_ANSWER_ID, -16, z ? (byte) 1 : (byte) 0, checksum(bArr)};
        enqueueRequest(new EcamRequest(2, bArr));
    }

    public void setPin(String str) {
        if (str.length() < 4) {
            DLog.e(TAG, "Pin too short!");
            return;
        }
        byte[] bArr = {Ascii.CR, 8, -79, -16, (byte) str.charAt(0), (byte) str.charAt(1), (byte) str.charAt(2), (byte) str.charAt(3), checksum(bArr)};
        enqueueRequest(new EcamRequest(2, bArr));
    }

    public void setHour(int i, int i2) {
        byte[] bArr = {Ascii.CR, 6, SET_TIME_ANSWER, -16, (byte) i, (byte) i2, checksum(bArr)};
        enqueueRequest(new EcamRequest(2, bArr));
    }

    /* JADX INFO: Access modifiers changed from: private */
    public void enqueueInBackground(EcamRequest ecamRequest) {
        synchronized (this.mRequestQueue) {
            this.mRequestQueue.add(ecamRequest);
            this.mRequestQueue.notify();
        }
    }

    /* JADX INFO: Access modifiers changed from: private */
    public void updateReceivedInBackground(byte[] bArr) {
        byte b;
        DLog.test(TAG, "updateReceived: " + Utils.byteArrayToHex(bArr));
        byte b2 = bArr[2];
        synchronized (this.mRequestQueue) {
            if (this.currentRequest != null) {
                if (!Arrays.equals(this.currentRequest.getRequest(), bArr)) {
                    this.response = null;
                    this.requestTimeoutRetries = 0;
                    this.mHandler.removeCallbacks(this.mTimeout);
                    this.mHandler.removeCallbacks(this.mBusy);
                    if (b2 != -31) {
                        this.requestBusyRetries = 0;
                        b = this.currentRequest.getRequest()[2];
                        DLog.d(TAG, "requestId: " + Utils.byteToHex(b) + " answerId: " + Utils.byteToHex(b2));
                        if (b != -126 && ((b != -95 || b2 != -107) && b2 != b)) {
                            this.mHandler.postDelayed(this.mResponseMismatch, 100L);
                        }
                        this.responseMismatchRetries = 0;
                        this.currentRequest = null;
                        this.mRequestQueue.notify();
                    } else {
                        DLog.d(TAG, "Busy response!");
                        this.mHandler.postDelayed(this.mBusy, 100L);
                    }
                } else {
                    DLog.e(TAG, "UPDATE EQUAL TO REQUEST!");
                }
            } else {
                DLog.e(TAG, "NO PENDING REQUESTS!");
            }
            b = -1;
        }
        if (b != -1) {
            if ((bArr[bArr.length - 1] ^ checksum(bArr)) == 0) {
                try {
                    decodeResponse(bArr);
                    return;
                } catch (Exception e) {
                    Log.e("EcamManager", e.getLocalizedMessage());
                    return;
                }
            }
            DLog.d(TAG, "Checksum KO!");
            this.mUpdatesListener.onRequestChecksumKo(bArr[2]);
        }
    }

    public void useConnectionManager(Activity activity) {
        this.mBleManager.useConnectionManager(activity);
    }

    /* JADX INFO: Access modifiers changed from: private */
    /* loaded from: classes2.dex */
    public class UpdateReceivedTask extends AsyncTask<byte[], Void, Void> {
        private WeakReference<EcamManager> mWeakReference;

        UpdateReceivedTask(EcamManager ecamManager) {
            this.mWeakReference = new WeakReference<>(ecamManager);
        }

        /* JADX INFO: Access modifiers changed from: protected */
        @Override // android.os.AsyncTask
        public Void doInBackground(byte[]... bArr) {
            if (this.mWeakReference.get() != null) {
                this.mWeakReference.get().updateReceivedInBackground(bArr[0]);
                return null;
            }
            return null;
        }
    }

    /* JADX INFO: Access modifiers changed from: private */
    /* loaded from: classes2.dex */
    public class EnqueueRequestTask extends AsyncTask<EcamRequest, Void, Void> {
        private WeakReference<EcamManager> mWeakReference;

        EnqueueRequestTask(EcamManager ecamManager) {
            this.mWeakReference = new WeakReference<>(ecamManager);
        }

        /* JADX INFO: Access modifiers changed from: protected */
        @Override // android.os.AsyncTask
        public Void doInBackground(EcamRequest... ecamRequestArr) {
            if (this.mWeakReference.get() != null) {
                this.mWeakReference.get().enqueueInBackground(ecamRequestArr[0]);
                return null;
            }
            return null;
        }
    }

    /* loaded from: classes2.dex */
    private class EcamScanCallback implements CustomLeScanCallback {
        private WeakReference<EcamManager> mWeakReference;

        @Override // it.delonghi.bluetooth.itf.CustomLeScanCallback
        public void scanStarted() {
        }

        EcamScanCallback(EcamManager ecamManager) {
            this.mWeakReference = new WeakReference<>(ecamManager);
        }

        @Override // it.delonghi.bluetooth.itf.CustomLeScanCallback
        public void scanStopped() {
            this.mWeakReference.get().scanFinished();
        }

        @Override // android.bluetooth.BluetoothAdapter.LeScanCallback
        public void onLeScan(BluetoothDevice bluetoothDevice, int i, byte[] bArr) {
            if (this.mWeakReference.get() != null) {
                DLog.d(EcamManager.TAG, "Ecam Machine found.");
                String str = EcamManager.TAG;
                DLog.d(str, "Name: " + bluetoothDevice.getName());
                String str2 = EcamManager.TAG;
                DLog.d(str2, "Address: " + bluetoothDevice.getAddress());
                String str3 = EcamManager.TAG;
                DLog.d(str3, "Bytes:" + Utils.byteArrayToHex(bArr));
                String name = bluetoothDevice.getName();
                if (name == null) {
                    name = BleUtils.parseAdertisedData(bArr).getName();
                }
                this.mWeakReference.get().addEcamMachine(bluetoothDevice.getAddress(), name);
            }
        }
    }

    /* loaded from: classes2.dex */
    private class EcamGattCallback extends BluetoothGattCallback {
        private final String TAG = EcamGattCallback.class.getName();
        private WeakReference<EcamManager> mWeakReference;

        EcamGattCallback(EcamManager ecamManager) {
            this.mWeakReference = new WeakReference<>(ecamManager);
        }

        @Override // android.bluetooth.BluetoothGattCallback
        public void onCharacteristicChanged(BluetoothGatt bluetoothGatt, BluetoothGattCharacteristic bluetoothGattCharacteristic) {
            if (this.mWeakReference.get() == null || !bluetoothGattCharacteristic.getUuid().toString().equalsIgnoreCase(Constants.TRANSFER_CHARACTERISTIC_UUID)) {
                return;
            }
            byte[] value = bluetoothGatt.getService(UUID.fromString(Constants.TRANSFER_SERVICE_UUID)).getCharacteristic(UUID.fromString(Constants.TRANSFER_CHARACTERISTIC_UUID)).getValue();
            if (this.mWeakReference.get().response == null) {
                this.mWeakReference.get().response = value;
            } else {
                byte[] bArr = EcamManager.this.response;
                int length = value.length + bArr.length;
                this.mWeakReference.get().response = new byte[length];
                for (int i = 0; i < length; i++) {
                    if (i < bArr.length) {
                        this.mWeakReference.get().response[i] = bArr[i];
                    } else {
                        this.mWeakReference.get().response[i] = value[i - bArr.length];
                    }
                }
            }
            if (EcamManager.this.isResponseComplete()) {
                this.mWeakReference.get().updateReceived(EcamManager.this.response);
            }
        }

        @Override // android.bluetooth.BluetoothGattCallback
        public void onCharacteristicRead(BluetoothGatt bluetoothGatt, BluetoothGattCharacteristic bluetoothGattCharacteristic, int i) {
            String str = this.TAG;
            DLog.i(str, "onCharacteristicRead " + i);
        }

        @Override // android.bluetooth.BluetoothGattCallback
        public void onCharacteristicWrite(BluetoothGatt bluetoothGatt, BluetoothGattCharacteristic bluetoothGattCharacteristic, int i) {
            String str = this.TAG;
            DLog.i(str, "onCharacteristicWrite " + i);
            if (bluetoothGattCharacteristic.getUuid().toString().equalsIgnoreCase(Constants.TRANSFER_CHARACTERISTIC_UUID)) {
                DLog.d(this.TAG, Utils.byteArrayToHex(bluetoothGattCharacteristic.getValue()));
                if (EcamManager.this.chunk != null) {
                    EcamManager.access$808(EcamManager.this);
                    if (EcamManager.this.nextChunkIdx >= EcamManager.this.chunk.length) {
                        EcamManager.this.chunk = null;
                        EcamManager.this.mHandler.postDelayed(EcamManager.this.mTimeout, 3000L);
                        return;
                    }
                    EcamManager ecamManager = EcamManager.this;
                    ecamManager.writeBytes(ecamManager.chunk[EcamManager.this.nextChunkIdx]);
                }
            }
        }

        @Override // android.bluetooth.BluetoothGattCallback
        public void onConnectionStateChange(BluetoothGatt bluetoothGatt, int i, int i2) {
            if (this.mWeakReference.get() != null) {
                if (i != 0) {
                    String str = this.TAG;
                    DLog.d(str, "Machine Disconnection, because BluetoothGatt is" + i);
                    this.mWeakReference.get().mConnectedEcamMachineAddress = null;
                    this.mWeakReference.get().mUpdatesListener.onMachineDisconnected(bluetoothGatt.getDevice().getAddress());
                    bluetoothGatt.close();
                } else if (i2 == 2) {
                    DLog.d(this.TAG, "Connected to GATT server.");
                    String address = bluetoothGatt.getDevice().getAddress();
                    DeLonghiManager.getInstance().setCurrentEcamMachineAddress(address);
                    this.mWeakReference.get().mConnectedEcamMachineAddress = address;
                    bluetoothGatt.discoverServices();
                } else if (i2 == 0) {
                    DLog.d(this.TAG, "Disconnected from GATT server.");
                    DeLonghiManager.getInstance().setCurrentEcamMachineAddress(EcamManager.this.mConnectedEcamMachineAddress);
                    this.mWeakReference.get().mConnectedEcamMachineAddress = null;
                    this.mWeakReference.get().mUpdatesListener.onMachineDisconnected(bluetoothGatt.getDevice() != null ? bluetoothGatt.getDevice().getAddress() : null);
                }
            }
        }

        @Override // android.bluetooth.BluetoothGattCallback
        public void onServicesDiscovered(BluetoothGatt bluetoothGatt, int i) {
            String str = this.TAG;
            DLog.i(str, "onServicesDiscovered received: " + Integer.toHexString(i));
            if (this.mWeakReference.get() != null) {
                if (i == 0) {
                    BluetoothGattService service = bluetoothGatt.getService(EcamManager.mEcamServiceUUIDs[0]);
                    if (service != null) {
                        this.mWeakReference.get().mEcamCharacteristic = service.getCharacteristic(EcamManager.mEcamCharacteristicUUID);
                        this.mWeakReference.get().mEcamCharacteristic.setWriteType(2);
                        bluetoothGatt.setCharacteristicNotification(this.mWeakReference.get().mEcamCharacteristic, true);
                        BluetoothGattDescriptor descriptor = this.mWeakReference.get().mEcamCharacteristic.getDescriptor(UUID.fromString("00002902-0000-1000-8000-00805f9b34fb"));
                        descriptor.setValue(BluetoothGattDescriptor.ENABLE_INDICATION_VALUE);
                        bluetoothGatt.writeDescriptor(descriptor);
                        return;
                    }
                    return;
                }
                this.mWeakReference.get().disconnectFromCurrentEcamMachine();
            }
        }

        @Override // android.bluetooth.BluetoothGattCallback
        public void onDescriptorWrite(BluetoothGatt bluetoothGatt, BluetoothGattDescriptor bluetoothGattDescriptor, int i) {
            String str = this.TAG;
            DLog.d(str, "onDescriptorWrite " + i);
            if (i == 0) {
                this.mWeakReference.get().mUpdatesListener.onMachineConnected(bluetoothGatt.getDevice().getAddress());
            } else {
                EcamManager.this.disconnectFromCurrentEcamMachine();
            }
        }

        @Override // android.bluetooth.BluetoothGattCallback
        public void onDescriptorRead(BluetoothGatt bluetoothGatt, BluetoothGattDescriptor bluetoothGattDescriptor, int i) {
            String str = this.TAG;
            DLog.d(str, "onDescriptorRead " + i);
        }
    }
}
