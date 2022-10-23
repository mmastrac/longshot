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
import android.os.Bundle;
import android.os.Handler;
import android.util.Base64;
import android.util.Log;
import com.google.android.exoplayer2.C;
import com.google.android.exoplayer2.extractor.ts.PsExtractor;
import com.google.common.base.Ascii;
import it.delonghi.Constants;
import it.delonghi.DeLonghi;
import it.delonghi.DeLonghiManager;
import it.delonghi.ayla.constant.AylaProperties;
import it.delonghi.bluetooth.BleManager;
import it.delonghi.bluetooth.BleUtils;
import it.delonghi.bluetooth.itf.CustomLeScanCallback;
import it.delonghi.ecam.itf.EcamUpdatesReceived;
import it.delonghi.ecam.model.EcamRequest;
import it.delonghi.ecam.model.Parameter;
import it.delonghi.ecam.model.RecipeData;
import it.delonghi.ecam.model.enums.BeverageTasteType;
import it.delonghi.ecam.model.enums.BeverageTasteValue;
import it.delonghi.ecam.model.enums.OperationTriggerId;
import it.delonghi.model.BeanSystem;
import it.delonghi.model.ParameterModel;
import it.delonghi.model.RecipeDefaults;
import it.delonghi.service.DeLonghiWifiConnectService;
import it.delonghi.utils.DLog;
import it.delonghi.utils.Utils;
import it.delonghi.utils.comparators.RequestPriorityComparator;
import java.lang.ref.WeakReference;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Iterator;
import java.util.PriorityQueue;
import java.util.UUID;
import org.objectweb.asm.Opcodes;

/* loaded from: classes2.dex */
public class EcamManagerV2 {
    public static final int ANSWER_ID_BYTE_IDX = 2;
    private static final byte BEAN_SYSTEM_READ_ANSWER_ID = -70;
    private static final byte BEAN_SYSTEM_SELECT_ANSWER_ID = -71;
    private static final byte BEAN_SYSTEM_WRITE_ANSWER_ID = -69;
    public static final byte BEVERAGE_DISPENSING_ANSWER_ID = -125;
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
    private static final byte PARAMETER_READ_EXT_ANSWER_ID = -95;
    public static final byte PARAMETER_WRITE_ANSWER_ID = -112;
    private static final byte PIN_ACTIVATION_ANSWER_ID = -80;
    public static final byte PIN_SET_ANSWER_ID = -79;
    private static final int PRIORITY_HIGH = 1;
    public static final int PRIORITY_LOW = 3;
    private static final int PRIORITY_NORMAL = 2;
    private static final byte PROFILES_NAME_READ_ANSWER_ID = -92;
    private static final byte PROFILES_NAME_WRITE_ANSWER_ID = -91;
    private static final byte PROFILE_SELECTION_ANSWER_ID = -87;
    private static final byte READ_PIN_ID = -46;
    public static final byte RECIPES_NAME_READ_ANSWER_ID = -86;
    public static final byte RECIPES_NAME_WRITE_ANSWER_ID = -85;
    public static final byte RECIPES_PRIORITY_READ_ANSWER_ID = -88;
    public static final byte RECIPES_QTY_READ_ANSWER_ID = -90;
    private static final byte REMOTE_CONTROL = 12;
    private static final byte SET_TIME_ANSWER = -30;
    public static final byte STATISTICS_READ_ANSWER_ID = -94;
    private static final int TIMEOUT = 1000;
    private byte[][] chunk;
    private EcamRequest currentRequest;
    private BleManager mBleManager;
    private String mConnectedEcamMachineAddress;
    private Context mContext;
    private BluetoothGattCharacteristic mEcamCharacteristic;
    private ArrayList<String> mEcamMachinesAddresses;
    private Handler mHandler;
    private final PriorityQueue<EcamRequest> mRequestQueue;
    private EcamUpdatesReceived mUpdatesListener;
    private byte[] response;
    private static final String TAG = EcamManagerV2.class.getName();
    private static final UUID[] mEcamServiceUUIDs = {UUID.fromString(Constants.TRANSFER_SERVICE_UUID)};
    private static final UUID mEcamCharacteristicUUID = UUID.fromString(Constants.TRANSFER_CHARACTERISTIC_UUID);
    private int nextChunkIdx = 0;
    private int requestTimeoutRetries = 2;
    private int requestBusyRetries = 0;
    private int responseMismatchRetries = 0;
    private boolean mIsWifi = false;
    private Runnable mTimeout = new Runnable() { // from class: it.delonghi.ecam.EcamManagerV2.1
        @Override // java.lang.Runnable
        public void run() {
            Log.d(EcamManagerV2.TAG, "Request timeout!");
            EcamManagerV2.this.response = null;
            synchronized (EcamManagerV2.this.mRequestQueue) {
                if (EcamManagerV2.this.currentRequest != null) {
                    if (EcamManagerV2.this.requestTimeoutRetries >= 5) {
                        EcamManagerV2.this.requestTimeoutRetries = 0;
                        byte[] request = EcamManagerV2.this.currentRequest.getRequest();
                        byte b = request[2];
                        String str = EcamManagerV2.TAG;
                        DLog.d(str, "BLE FLOW Request timeout for " + Utils.byteToHex(request[2]));
                        EcamManagerV2.this.mUpdatesListener.onRequestTimeout(b);
                    } else {
                        String str2 = EcamManagerV2.TAG;
                        DLog.d(str2, "BLE FLOW Timeout Retry for " + Utils.byteToHex(EcamManagerV2.this.currentRequest.getRequest()[2]));
                        EcamManagerV2.this.currentRequest.setPriority(1);
                        EcamManagerV2.this.mRequestQueue.add(EcamManagerV2.this.currentRequest);
                        EcamManagerV2.access$408(EcamManagerV2.this);
                    }
                    EcamManagerV2.this.currentRequest = null;
                    EcamManagerV2.this.mRequestQueue.notify();
                }
            }
        }
    };
    private Runnable mBusy = new Runnable() { // from class: it.delonghi.ecam.EcamManagerV2.2
        @Override // java.lang.Runnable
        public void run() {
            DLog.d(EcamManagerV2.TAG, "Busy retry");
            EcamManagerV2.this.response = null;
            synchronized (EcamManagerV2.this.mRequestQueue) {
                if (EcamManagerV2.this.currentRequest != null) {
                    if (EcamManagerV2.this.requestBusyRetries >= 10) {
                        EcamManagerV2.this.requestBusyRetries = 0;
                        EcamManagerV2.this.mUpdatesListener.onRequestTimeout(EcamManagerV2.this.currentRequest.getRequest()[2]);
                    } else {
                        EcamManagerV2.this.currentRequest.setPriority(1);
                        EcamManagerV2.this.mRequestQueue.add(EcamManagerV2.this.currentRequest);
                        EcamManagerV2.access$608(EcamManagerV2.this);
                    }
                    EcamManagerV2.this.currentRequest = null;
                    EcamManagerV2.this.mRequestQueue.notify();
                }
            }
        }
    };
    private Runnable mResponseMismatch = new Runnable() { // from class: it.delonghi.ecam.EcamManagerV2.3
        @Override // java.lang.Runnable
        public void run() {
            DLog.d(EcamManagerV2.TAG, "Answer mismatch retry");
            EcamManagerV2.this.response = null;
            synchronized (EcamManagerV2.this.mRequestQueue) {
                if (EcamManagerV2.this.currentRequest != null) {
                    if (EcamManagerV2.this.responseMismatchRetries >= 5) {
                        EcamManagerV2.this.responseMismatchRetries = 0;
                        EcamManagerV2.this.mUpdatesListener.onRequestTimeout(EcamManagerV2.this.currentRequest.getRequest()[2]);
                    } else {
                        EcamManagerV2.this.currentRequest.setPriority(1);
                        EcamManagerV2.this.mRequestQueue.add(EcamManagerV2.this.currentRequest);
                        EcamManagerV2.access$708(EcamManagerV2.this);
                    }
                    EcamManagerV2.this.currentRequest = null;
                    EcamManagerV2.this.mRequestQueue.notify();
                }
            }
        }
    };
    private EcamScanCallback mEcamScanCallback = new EcamScanCallback(this);
    private EcamGattCallback mEcamGattCallback = new EcamGattCallback(this);

    static /* synthetic */ int access$1408(EcamManagerV2 ecamManagerV2) {
        int i = ecamManagerV2.nextChunkIdx;
        ecamManagerV2.nextChunkIdx = i + 1;
        return i;
    }

    static /* synthetic */ int access$408(EcamManagerV2 ecamManagerV2) {
        int i = ecamManagerV2.requestTimeoutRetries;
        ecamManagerV2.requestTimeoutRetries = i + 1;
        return i;
    }

    static /* synthetic */ int access$608(EcamManagerV2 ecamManagerV2) {
        int i = ecamManagerV2.requestBusyRetries;
        ecamManagerV2.requestBusyRetries = i + 1;
        return i;
    }

    static /* synthetic */ int access$708(EcamManagerV2 ecamManagerV2) {
        int i = ecamManagerV2.responseMismatchRetries;
        ecamManagerV2.responseMismatchRetries = i + 1;
        return i;
    }

    public EcamManagerV2(Context context, EcamUpdatesReceived ecamUpdatesReceived) {
        this.mContext = context;
        this.mUpdatesListener = ecamUpdatesReceived;
        this.mHandler = new Handler();
        BleManager bleManager = new BleManager(context);
        this.mBleManager = bleManager;
        bleManager.registerBleScanListener(this.mEcamScanCallback);
        this.mBleManager.registerGattListener(this.mEcamGattCallback);
        this.mHandler = new Handler();
        this.mRequestQueue = new PriorityQueue<>(20, new RequestPriorityComparator());
        new Thread(new Runnable() { // from class: it.delonghi.ecam.-$$Lambda$EcamManagerV2$tPq7RfM2iZobSalVYnf4Os_qlJA
            @Override // java.lang.Runnable
            public final void run() {
                EcamManagerV2.this.lambda$new$0$EcamManagerV2();
            }
        }).start();
    }

    public /* synthetic */ void lambda$new$0$EcamManagerV2() {
        while (true) {
            byte[] bArr = null;
            synchronized (this.mRequestQueue) {
                try {
                    this.mRequestQueue.wait();
                } catch (InterruptedException e) {
                    String str = TAG;
                    DLog.e(str, "BLE FLOW InterruptedException!" + e.getMessage());
                }
                if (this.currentRequest == null && this.mRequestQueue.size() > 0) {
                    EcamRequest poll = this.mRequestQueue.poll();
                    this.currentRequest = poll;
                    bArr = poll.getRequest();
                }
                if (bArr != null) {
                    if (this.mIsWifi) {
                        if (sendCommand(bArr)) {
                            this.mHandler.postDelayed(this.mTimeout, 3000L);
                        } else {
                            this.mHandler.post(this.mTimeout);
                        }
                    } else if (bArr.length > BleManager.BLE_PACKET_MAX_SIZE) {
                        this.nextChunkIdx = 0;
                        byte[][] divideArray = Utils.divideArray(bArr, BleManager.BLE_PACKET_MAX_SIZE);
                        this.chunk = divideArray;
                        writeBytes(divideArray[this.nextChunkIdx]);
                    } else if (writeBytes(bArr)) {
                        this.mHandler.postDelayed(this.mTimeout, 1000L);
                    } else {
                        this.mHandler.post(this.mTimeout);
                    }
                }
            }
        }
    }

    public void useConnectionManager(Activity activity) {
        this.mBleManager.useConnectionManager(activity);
    }

    /* loaded from: classes2.dex */
    private class EcamScanCallback implements CustomLeScanCallback {
        private WeakReference<EcamManagerV2> mWeakReference;

        @Override // it.delonghi.bluetooth.itf.CustomLeScanCallback
        public void scanStarted() {
        }

        EcamScanCallback(EcamManagerV2 ecamManagerV2) {
            this.mWeakReference = new WeakReference<>(ecamManagerV2);
        }

        @Override // it.delonghi.bluetooth.itf.CustomLeScanCallback
        public void scanStopped() {
            this.mWeakReference.get().scanFinished();
        }

        @Override // android.bluetooth.BluetoothAdapter.LeScanCallback
        public void onLeScan(BluetoothDevice bluetoothDevice, int i, byte[] bArr) {
            if (this.mWeakReference.get() != null) {
                DLog.d(EcamManagerV2.TAG, "Ecam Machine found.");
                String str = EcamManagerV2.TAG;
                DLog.d(str, "Name: " + bluetoothDevice.getName());
                String str2 = EcamManagerV2.TAG;
                DLog.d(str2, "Address: " + bluetoothDevice.getAddress());
                String str3 = EcamManagerV2.TAG;
                DLog.d(str3, "Bytes:" + Utils.byteArrayToHex(bArr));
                String name = bluetoothDevice.getName();
                if (name == null) {
                    name = BleUtils.parseAdertisedData(bArr).getName();
                }
                this.mWeakReference.get().addEcamMachine(bluetoothDevice.getAddress(), name);
            }
        }
    }

    public BluetoothDevice getEcamDevice(String str) {
        return this.mBleManager.getBluetoothDevice(str);
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

    public void updateReceivedInBackground(byte[] bArr) {
        byte b;
        DLog.e(TAG, "updateReceived: " + Utils.byteArrayToHex(bArr));
        byte b2 = bArr[2];
        synchronized (this.mRequestQueue) {
            if (this.currentRequest != null) {
                if (!Arrays.equals(this.currentRequest.getRequest(), bArr)) {
                    this.response = null;
                    this.requestTimeoutRetries = 0;
                    this.mHandler.removeCallbacks(this.mTimeout);
                    this.mHandler.removeCallbacks(this.mBusy);
                    if (b2 == -70) {
                        this.requestBusyRetries = 0;
                        b = this.currentRequest.getRequest()[2];
                        String str = TAG;
                        StringBuilder sb = new StringBuilder();
                        sb.append("BLE FLOW requestId: ");
                        sb.append(Utils.byteToHex(b));
                        sb.append(" answerId: ");
                        byte b3 = b2;
                        sb.append(Utils.byteToHex(b3));
                        DLog.e(str, sb.toString());
                        DLog.e(TAG, "BLE FLOW request: " + Utils.byteArrayToHex(this.currentRequest.getRequest()) + " answer: " + Utils.byteArrayToHex(bArr));
                        boolean equalsIgnoreCase = Utils.byteToHex(b3).equalsIgnoreCase("ba");
                        boolean z = b2 == b;
                        boolean z2 = this.currentRequest.getRequest()[4] == bArr[4];
                        if (equalsIgnoreCase && z && z2) {
                            this.responseMismatchRetries = 0;
                            this.currentRequest = null;
                            this.mRequestQueue.notify();
                        } else if (!this.mIsWifi) {
                            this.mHandler.postDelayed(this.mResponseMismatch, 100L);
                        }
                    } else if (b2 != -31) {
                        this.requestBusyRetries = 0;
                        b = this.currentRequest.getRequest()[2];
                        if (b != -125 && ((b != -95 || b2 != -107) && b2 != b)) {
                            if (!this.mIsWifi) {
                                this.mHandler.postDelayed(this.mResponseMismatch, 100L);
                            }
                        }
                        DLog.e(TAG, "BLE FLOW correct answer for requestId: " + Utils.byteToHex(b) + " answerId: " + Utils.byteToHex(b2));
                        this.responseMismatchRetries = 0;
                        this.currentRequest = null;
                        this.mRequestQueue.notify();
                    } else {
                        DLog.e(TAG, "BLE FLOW Busy response!");
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
            byte b4 = bArr[bArr.length - 2];
            byte b5 = bArr[bArr.length - 1];
            int checksum = checksum(bArr);
            int i = b5 ^ ((byte) (checksum & 255));
            if ((b4 ^ ((byte) ((checksum >> 8) & 255))) == 0 && i == 0) {
                decodeResponse(bArr);
                return;
            }
            DLog.e(TAG, "Checksum KO!");
            this.mUpdatesListener.onRequestChecksumKo(bArr[2]);
        }
    }

    /*  JADX ERROR: JadxRuntimeException in pass: RegionMakerVisitor
        jadx.core.utils.exceptions.JadxRuntimeException: Failed to find switch 'out' block
        	at jadx.core.dex.visitors.regions.RegionMaker.processSwitch(RegionMaker.java:817)
        	at jadx.core.dex.visitors.regions.RegionMaker.traverse(RegionMaker.java:160)
        	at jadx.core.dex.visitors.regions.RegionMaker.makeRegion(RegionMaker.java:94)
        	at jadx.core.dex.visitors.regions.RegionMaker.processSwitch(RegionMaker.java:856)
        	at jadx.core.dex.visitors.regions.RegionMaker.traverse(RegionMaker.java:160)
        	at jadx.core.dex.visitors.regions.RegionMaker.makeRegion(RegionMaker.java:94)
        	at jadx.core.dex.visitors.regions.RegionMaker.processIf(RegionMaker.java:730)
        	at jadx.core.dex.visitors.regions.RegionMaker.traverse(RegionMaker.java:155)
        	at jadx.core.dex.visitors.regions.RegionMaker.makeRegion(RegionMaker.java:94)
        	at jadx.core.dex.visitors.regions.RegionMaker.processIf(RegionMaker.java:730)
        	at jadx.core.dex.visitors.regions.RegionMaker.traverse(RegionMaker.java:155)
        	at jadx.core.dex.visitors.regions.RegionMaker.makeRegion(RegionMaker.java:94)
        	at jadx.core.dex.visitors.regions.RegionMaker.processIf(RegionMaker.java:730)
        	at jadx.core.dex.visitors.regions.RegionMaker.traverse(RegionMaker.java:155)
        	at jadx.core.dex.visitors.regions.RegionMaker.makeRegion(RegionMaker.java:94)
        	at jadx.core.dex.visitors.regions.RegionMaker.processIf(RegionMaker.java:730)
        	at jadx.core.dex.visitors.regions.RegionMaker.traverse(RegionMaker.java:155)
        	at jadx.core.dex.visitors.regions.RegionMaker.makeRegion(RegionMaker.java:94)
        	at jadx.core.dex.visitors.regions.RegionMaker.processIf(RegionMaker.java:730)
        	at jadx.core.dex.visitors.regions.RegionMaker.traverse(RegionMaker.java:155)
        	at jadx.core.dex.visitors.regions.RegionMaker.makeRegion(RegionMaker.java:94)
        	at jadx.core.dex.visitors.regions.RegionMaker.processIf(RegionMaker.java:730)
        	at jadx.core.dex.visitors.regions.RegionMaker.traverse(RegionMaker.java:155)
        	at jadx.core.dex.visitors.regions.RegionMaker.makeRegion(RegionMaker.java:94)
        	at jadx.core.dex.visitors.regions.RegionMaker.processIf(RegionMaker.java:730)
        	at jadx.core.dex.visitors.regions.RegionMaker.traverse(RegionMaker.java:155)
        	at jadx.core.dex.visitors.regions.RegionMaker.makeRegion(RegionMaker.java:94)
        	at jadx.core.dex.visitors.regions.RegionMaker.processIf(RegionMaker.java:730)
        	at jadx.core.dex.visitors.regions.RegionMaker.traverse(RegionMaker.java:155)
        	at jadx.core.dex.visitors.regions.RegionMaker.makeRegion(RegionMaker.java:94)
        	at jadx.core.dex.visitors.regions.RegionMaker.processIf(RegionMaker.java:730)
        	at jadx.core.dex.visitors.regions.RegionMaker.traverse(RegionMaker.java:155)
        	at jadx.core.dex.visitors.regions.RegionMaker.makeRegion(RegionMaker.java:94)
        	at jadx.core.dex.visitors.regions.RegionMaker.processIf(RegionMaker.java:730)
        	at jadx.core.dex.visitors.regions.RegionMaker.traverse(RegionMaker.java:155)
        	at jadx.core.dex.visitors.regions.RegionMaker.makeRegion(RegionMaker.java:94)
        	at jadx.core.dex.visitors.regions.RegionMakerVisitor.visit(RegionMakerVisitor.java:52)
        */
    private void decodeResponse(byte[] r13) {
        /*
            Method dump skipped, instructions count: 742
            To view this dump change 'Code comments level' option to 'DEBUG'
        */
        throw new UnsupportedOperationException("Method not decompiled: it.delonghi.ecam.EcamManagerV2.decodeResponse(byte[]):void");
    }

    public static String bytesToHexString(byte[] bArr) {
        StringBuilder sb = new StringBuilder();
        int length = bArr.length;
        for (int i = 0; i < length; i++) {
            sb.append(String.format("%02x", Integer.valueOf(bArr[i] & 255)));
        }
        return sb.toString();
    }

    public static RecipeDefaults getDefaultMinMaxQty(byte[] bArr) {
        int unsignedIntFromByte;
        int i;
        int i2;
        DLog.e(TAG, "getDefaultMinMaxQty");
        int i3 = bArr[1] & 255;
        int unsignedIntFromByte2 = Utils.getUnsignedIntFromByte(bArr[4]);
        int i4 = (i3 - 6) / 4;
        String str = TAG;
        DLog.d(str, "txLength :" + i3);
        String str2 = TAG;
        DLog.d(str2, "recipeLength :" + i4);
        ArrayList arrayList = new ArrayList();
        int i5 = 0;
        for (int i6 = 0; i6 < i4; i6++) {
            int i7 = (i6 * 4) + i5;
            int i8 = i7 + 8;
            if (i3 <= i8) {
                break;
            }
            ParameterModel parameterModel = new ParameterModel();
            int unsignedIntFromByte3 = Utils.getUnsignedIntFromByte(bArr[i7 + 5]);
            String str3 = TAG;
            DLog.d(str3, "pramId :" + unsignedIntFromByte3);
            if (Utils.isTwoBytesShort(unsignedIntFromByte3)) {
                unsignedIntFromByte = Utils.twoBytesToShort(bArr[i7 + 6], bArr[i7 + 7]);
                i = Utils.twoBytesToShort(bArr[i8], bArr[i7 + 9]);
                i2 = Utils.twoBytesToShort(bArr[i7 + 10], bArr[i7 + 11]);
                i5 += 3;
            } else {
                unsignedIntFromByte = Utils.getUnsignedIntFromByte(bArr[i7 + 6]);
                int unsignedIntFromByte4 = Utils.getUnsignedIntFromByte(bArr[i7 + 7]);
                int unsignedIntFromByte5 = Utils.getUnsignedIntFromByte(bArr[i8]);
                i = unsignedIntFromByte4;
                i2 = unsignedIntFromByte5;
            }
            parameterModel.setId(unsignedIntFromByte3);
            parameterModel.setMinValue(unsignedIntFromByte);
            parameterModel.setMaxValue(i2);
            parameterModel.setDefValue(i);
            arrayList.add(parameterModel);
        }
        return new RecipeDefaults(unsignedIntFromByte2, -1, -1, -1, -1, -1, -1, -1, arrayList, false);
    }

    public static RecipeData getRecipeDataFromByteArray(byte[] bArr) {
        int unsignedIntFromByte;
        boolean z;
        int i = bArr[1] & 255;
        int unsignedIntFromByte2 = Utils.getUnsignedIntFromByte(bArr[5]);
        new ArrayList();
        int i2 = (i - 7) / 2;
        DLog.e(TAG, "getRecipeDataFromByteArray - Beverage ID: " + unsignedIntFromByte2 + " ingredientsLength :" + i2);
        ArrayList<ParameterModel> arrayList = new ArrayList<>();
        int i3 = 0;
        for (int i4 = 0; i4 < i2; i4++) {
            int i5 = (i4 * 2) + i3;
            int i6 = i5 + 7;
            if (i <= i6) {
                break;
            }
            int unsignedIntFromByte3 = Utils.getUnsignedIntFromByte(bArr[i5 + 6]);
            if (Utils.isTwoBytesShort(unsignedIntFromByte3)) {
                unsignedIntFromByte = Utils.twoBytesToShort(bArr[i6], bArr[i5 + 8]);
                i3++;
            } else {
                unsignedIntFromByte = Utils.getUnsignedIntFromByte(bArr[i6]);
            }
            Iterator<ParameterModel> it2 = arrayList.iterator();
            while (true) {
                if (!it2.hasNext()) {
                    z = false;
                    break;
                }
                ParameterModel next = it2.next();
                if (next.getId() == unsignedIntFromByte3) {
                    next.setDefValue(unsignedIntFromByte);
                    z = true;
                    break;
                }
            }
            if (!z) {
                ParameterModel parameterModel = new ParameterModel();
                parameterModel.setId(unsignedIntFromByte3);
                parameterModel.setDefValue(unsignedIntFromByte);
                arrayList.add(parameterModel);
            }
        }
        RecipeData recipeData = new RecipeData(unsignedIntFromByte2);
        recipeData.setIngredients(arrayList);
        return recipeData;
    }

    public static RecipeData getDefaultRecipeDataFromByteArray(byte[] bArr) {
        int unsignedIntFromByte;
        boolean z;
        int i = bArr[1] & 255;
        int unsignedIntFromByte2 = Utils.getUnsignedIntFromByte(bArr[4]);
        new ArrayList();
        int i2 = (i - 7) / 2;
        DLog.e(TAG, "getRecipeDataFromByteArray - Beverage ID: " + unsignedIntFromByte2 + " ingredientsLength :" + i2);
        ArrayList<ParameterModel> arrayList = new ArrayList<>();
        int i3 = 0;
        for (int i4 = 0; i4 < i2; i4++) {
            int i5 = (i4 * 2) + i3;
            int i6 = i5 + 7;
            if (i <= i6) {
                break;
            }
            int unsignedIntFromByte3 = Utils.getUnsignedIntFromByte(bArr[i5 + 5]);
            if (Utils.isTwoBytesShort(unsignedIntFromByte3)) {
                unsignedIntFromByte = Utils.twoBytesToShort(bArr[i5 + 6], bArr[i6]);
                i3++;
            } else {
                unsignedIntFromByte = Utils.getUnsignedIntFromByte(bArr[i5 + 6]);
            }
            Iterator<ParameterModel> it2 = arrayList.iterator();
            while (true) {
                if (!it2.hasNext()) {
                    z = false;
                    break;
                }
                ParameterModel next = it2.next();
                if (next.getId() == unsignedIntFromByte3) {
                    next.setDefValue(unsignedIntFromByte);
                    z = true;
                    break;
                }
            }
            if (!z) {
                ParameterModel parameterModel = new ParameterModel();
                parameterModel.setId(unsignedIntFromByte3);
                parameterModel.setDefValue(unsignedIntFromByte);
                arrayList.add(parameterModel);
            }
        }
        RecipeData recipeData = new RecipeData(unsignedIntFromByte2);
        recipeData.setIngredients(arrayList);
        return recipeData;
    }

    public static ArrayList<Parameter> getParametersFromByte(byte[] bArr) {
        Parameter parameter;
        DLog.e(TAG, "getParametersFromByte");
        int i = ((bArr[1] & 255) - 7) / 4;
        ArrayList<Parameter> arrayList = new ArrayList<>(i);
        int twoBytesToShort = Utils.twoBytesToShort(bArr[4], bArr[5]);
        for (int i2 = 0; i2 < i; i2++) {
            byte[] bArr2 = new byte[4];
            System.arraycopy(bArr, (i2 * 4) + 6, bArr2, 0, 4);
            arrayList.add(new Parameter(twoBytesToShort, bArr2));
            twoBytesToShort++;
            DLog.e(TAG, "getParametersFromByte = " + parameter.getIndex() + " value " + ((int) parameter.getLongValue()));
        }
        return arrayList;
    }

    public static void readParametersFromByte(byte[] bArr) {
        int i = ((bArr[1] & 255) - 7) / 4;
        ArrayList arrayList = new ArrayList(i);
        int twoBytesToShort = Utils.twoBytesToShort(bArr[4], bArr[5]);
        for (int i2 = 0; i2 < i; i2++) {
            byte[] bArr2 = new byte[4];
            System.arraycopy(bArr, (i2 * 4) + 6, bArr2, 0, 4);
            arrayList.add(new Parameter(twoBytesToShort, bArr2));
            twoBytesToShort++;
            DLog.e(TAG, "##Param " + twoBytesToShort + " type " + Utils.byteToHex(bArr[2]));
        }
    }

    public static BeanSystem loadBeanSystems(byte[] bArr) {
        float f;
        DLog.e(TAG, "loadBeanSystems");
        if (DeLonghi.getInstance().getConnectService() instanceof DeLonghiWifiConnectService) {
            boolean isStriker = ((DeLonghiWifiConnectService) DeLonghi.getInstance().getConnectService()).isStriker();
            try {
                try {
                    byte[] bArr2 = new byte[40];
                    System.arraycopy(bArr, 5, bArr2, 0, 40);
                    String byteArrayToString = Utils.isByteArrayAllZeros(bArr2) ? "" : Utils.byteArrayToString(Utils.trim(bArr2), C.UTF16_NAME);
                    boolean z = Utils.getUnsignedIntFromByte(bArr[49]) != 1;
                    boolean z2 = Utils.getUnsignedIntFromByte(bArr[50]) != 0;
                    float unsignedIntFromByte = Utils.getUnsignedIntFromByte(bArr[45]);
                    if (isStriker) {
                        if (unsignedIntFromByte == 13.0f) {
                            unsignedIntFromByte = 14.0f;
                        }
                        f = (float) (unsignedIntFromByte / 2.0d);
                    } else {
                        f = unsignedIntFromByte;
                    }
                    return new BeanSystem(bArr[4], byteArrayToString, "", z2, z, f, Utils.getUnsignedIntFromByte(bArr[46]), Utils.getUnsignedIntFromByte(bArr[47]), Utils.getOptimalId(bArr[4]));
                } catch (Exception unused) {
                }
            } catch (Exception unused2) {
                return new BeanSystem(bArr[4], "", "", false, true, 0.0f, 0, 0, 200);
            }
        }
        return null;
    }

    public static int[] readPriorities(byte[] bArr) {
        DLog.e(TAG, "readPriorities");
        int[] iArr = new int[0];
        if (bArr == null || bArr.length <= 0) {
            return iArr;
        }
        Utils.byteToInt(bArr[4]);
        int i = (bArr[1] & 255) - 6;
        int[] iArr2 = new int[i];
        for (int i2 = 0; i2 < i; i2++) {
            iArr2[i2] = Utils.byteToInt(bArr[i2 + 5]);
        }
        return iArr2;
    }

    public static boolean readRemoteControl(byte[] bArr) {
        DLog.e(TAG, "readRemoteControl");
        return Utils.byteToInt(bArr[9]) == 3;
    }

    public static Bundle readProfiles(byte[] bArr) {
        DLog.e(TAG, "readProfiles");
        Bundle bundle = new Bundle();
        int byteToInt = Utils.byteToInt(bArr[1]);
        int i = DeLonghiManager.getInstance().CONNECTION_TYPE.equalsIgnoreCase(DeLonghiManager.getInstance().CONNECTION_WIFI) ? 6 : 4;
        int i2 = (byteToInt - (i + 1)) / 21;
        if (bArr.length < 10) {
            Utils.byteArrayToString(Utils.trim(bArr), C.UTF16_NAME);
            for (byte b : bArr) {
                Integer.valueOf(Utils.byteToInt(b));
            }
        }
        ArrayList<String> arrayList = new ArrayList<>(i2);
        ArrayList<Integer> arrayList2 = new ArrayList<>(i2);
        for (int i3 = 0; i3 < i2; i3++) {
            byte[] bArr2 = new byte[20];
            int i4 = i3 * 21;
            System.arraycopy(bArr, i + i4, bArr2, 0, 20);
            String str = null;
            if (!Utils.isByteArrayAllZeros(bArr2)) {
                str = Utils.byteArrayToString(Utils.trim(bArr2), C.UTF16_NAME);
            }
            Integer valueOf = Integer.valueOf(Utils.byteToInt(bArr[i + 20 + i4]));
            arrayList.add(str);
            arrayList2.add(valueOf);
        }
        bundle.putStringArrayList(Constants.NAMES_EXTRA, arrayList);
        bundle.putIntegerArrayList(Constants.ICONS_EXTRA, arrayList2);
        return bundle;
    }

    public static Bundle readProfilesStriker(byte[] bArr) {
        DLog.e(TAG, "readProfiles");
        Bundle bundle = new Bundle();
        int byteToInt = Utils.byteToInt(bArr[1]);
        int i = DeLonghiManager.getInstance().CONNECTION_TYPE.equalsIgnoreCase(DeLonghiManager.getInstance().CONNECTION_WIFI) ? 6 : 4;
        int i2 = (byteToInt - (i + 1)) / 22;
        if (bArr.length < 10) {
            Utils.byteArrayToString(Utils.trim(bArr), C.UTF16_NAME);
            for (byte b : bArr) {
                Integer.valueOf(Utils.byteToInt(b));
            }
        }
        ArrayList<String> arrayList = new ArrayList<>(i2);
        ArrayList<Integer> arrayList2 = new ArrayList<>(i2);
        ArrayList<Integer> arrayList3 = new ArrayList<>(i2);
        for (int i3 = 0; i3 < i2; i3++) {
            byte[] bArr2 = new byte[20];
            int i4 = i3 * 22;
            System.arraycopy(bArr, i + i4, bArr2, 0, 20);
            String str = null;
            if (!Utils.isByteArrayAllZeros(bArr2)) {
                str = Utils.byteArrayToString(Utils.trim(bArr2), C.UTF16_NAME);
            }
            int i5 = i + 20 + i4;
            Integer valueOf = Integer.valueOf(Utils.byteToInt(bArr[i5]));
            Integer valueOf2 = Integer.valueOf(Utils.byteToInt(bArr[i5 + 1]));
            arrayList.add(str);
            arrayList2.add(valueOf);
            arrayList3.add(valueOf2);
        }
        bundle.putStringArrayList(Constants.NAMES_EXTRA, arrayList);
        bundle.putIntegerArrayList(Constants.ICONS_EXTRA, arrayList2);
        bundle.putIntegerArrayList(Constants.MUGS_EXTRA, arrayList3);
        return bundle;
    }

    public void getMonitorMode(int i) {
        String str = TAG;
        DLog.e(str, "getMonitorMode  dataN" + i);
        enqueueRequest(new EcamRequest(2, getByteMonitorMode(i)));
    }

    public static byte[] getByteMonitorMode(int i) {
        String str = TAG;
        DLog.e(str, "getByteMonitorMode  dataN" + i);
        byte[] bArr = new byte[6];
        bArr[0] = Ascii.CR;
        bArr[1] = 5;
        if (i == 0) {
            bArr[2] = DATA_0_ANSWER_ID;
        } else if (i == 1) {
            bArr[2] = DATA_1_ANSWER_ID;
        } else if (i == 2) {
            bArr[2] = 117;
        }
        bArr[3] = Ascii.SI;
        int checksum = checksum(bArr);
        bArr[4] = (byte) ((checksum >> 8) & 255);
        bArr[5] = (byte) (checksum & 255);
        return bArr;
    }

    public static int checksum(byte[] bArr) {
        int i = 7439;
        for (int i2 = 0; i2 < bArr.length - 2; i2++) {
            int i3 = (((i << 8) | (i >>> 8)) & 65535) ^ (bArr[i2] & 255);
            int i4 = i3 ^ ((i3 & 255) >> 4);
            int i5 = i4 ^ ((i4 << 12) & 65535);
            i = i5 ^ (((i5 & 255) << 5) & 65535);
        }
        return i & 65535;
    }

    public void checksumVerification() {
        int checksum = checksum(r0);
        byte[] bArr = {Ascii.CR, 5, CHECKSUM_ANSWER_ID, -16, (byte) ((checksum >> 8) & 255), (byte) (checksum & 255)};
        enqueueRequest(new EcamRequest(2, bArr));
    }

    public void dispenseBeverage(int i, int i2, OperationTriggerId operationTriggerId, ArrayList<ParameterModel> arrayList, BeverageTasteValue beverageTasteValue, BeverageTasteType beverageTasteType, Boolean bool) {
        String str = TAG;
        DLog.e(str, "dispenseBeverage  beverageId:" + i);
        enqueueRequest(new EcamRequest(2, dispenseBeveragePacket(i, i2, operationTriggerId, arrayList, beverageTasteType, bool)));
    }

    public static byte[] dispenseBeveragePacket(int i, int i2, OperationTriggerId operationTriggerId, ArrayList<ParameterModel> arrayList, BeverageTasteType beverageTasteType, Boolean bool) {
        int i3;
        int i4;
        ParameterModel next;
        DLog.e(TAG, "dispenseBeveragePacket  beverageId:" + i + " profileId:" + i2);
        int i5 = 0;
        if (arrayList != null) {
            Iterator<ParameterModel> it2 = arrayList.iterator();
            i3 = 0;
            while (it2.hasNext()) {
                ParameterModel next2 = it2.next();
                if (next2.getId() < 23 || next2.getId() == 28) {
                    if (bool.booleanValue() || i != 200 || next2.getId() != 2) {
                        i3 = Utils.isTwoBytesShort(next2.getId()) ? i3 + 3 : i3 + 2;
                    }
                }
            }
        } else {
            i3 = 0;
        }
        byte[] bArr = new byte[i3 + 9];
        bArr[0] = Ascii.CR;
        bArr[1] = (byte) (i3 + 8);
        bArr[2] = BEVERAGE_DISPENSING_ANSWER_ID;
        bArr[3] = -16;
        bArr[4] = (byte) i;
        if (bool.booleanValue()) {
            bArr[5] = (byte) (operationTriggerId.getValue() | 128);
        } else {
            bArr[5] = operationTriggerId.getValue();
        }
        int i6 = -2;
        if (arrayList != null) {
            Iterator<ParameterModel> it3 = arrayList.iterator();
            loop1: while (true) {
                i4 = 0;
                while (it3.hasNext()) {
                    next = it3.next();
                    if (next.getId() < 23 || next.getId() == 28) {
                        if (bool.booleanValue() || i != 200 || next.getId() != 2) {
                            i6 = i6 + 2 + i4;
                            bArr[i6 + 6] = (byte) next.getId();
                            if (Utils.isTwoBytesShort(next.getId())) {
                                bArr[i6 + 7] = (byte) (next.getDefValue() >> 8);
                                bArr[i6 + 8] = (byte) next.getDefValue();
                                i4 = 1;
                            }
                        }
                    }
                }
                bArr[i6 + 7] = (byte) next.getDefValue();
            }
            i5 = i4;
        }
        int i7 = i6 + i5;
        bArr[i7 + 8] = (byte) ((i2 << 2) | beverageTasteType.getValue());
        int checksum = checksum(bArr);
        bArr[i7 + 9] = (byte) ((checksum >> 8) & 255);
        bArr[i7 + 10] = (byte) (checksum & 255);
        return bArr;
    }

    public void dispenseTEstBeverage() {
        int checksum = checksum(r1);
        byte[] bArr = {Ascii.CR, 17, BEVERAGE_DISPENSING_ANSWER_ID, -16, 1, 1, 1, 0, 40, 2, 3, 8, 0, Ascii.ESC, 4, Ascii.DC2, (byte) ((checksum >> 8) & 255), (byte) (checksum & 255)};
        enqueueRequest(new EcamRequest(2, bArr));
    }

    public void getParameters(int i, int i2) {
        String str = TAG;
        DLog.e(str, "getParameters  paramAddress:" + i + "paramsNumber:" + i2);
        getParameters(i, i2, 2);
    }

    private void getStatParameters(int i, int i2, int i3) {
        DLog.e(TAG, "getStatParameters  paramAddress:" + i + "paramsNumber:" + i2 + "requestPriority:" + i3);
        if (i2 > 10) {
            i2 = 10;
        }
        int checksum = checksum(r0);
        byte[] bArr = {Ascii.CR, 8, STATISTICS_READ_ANSWER_ID, Ascii.SI, (byte) (i >> 8), (byte) i, (byte) i2, (byte) ((checksum >> 8) & 255), (byte) (checksum & 255)};
        enqueueRequest(new EcamRequest(i3, bArr));
    }

    public static byte[] getParametersPacket(int i, int i2) {
        if (i2 > 10) {
            i2 = 10;
        }
        byte[] bArr = new byte[9];
        byte b = (byte) (i >> 8);
        byte b2 = (byte) i;
        bArr[0] = Ascii.CR;
        bArr[1] = 8;
        bArr[2] = (byte) (i2 > 4 ? Opcodes.IF_ICMPLT : Opcodes.FCMPL);
        bArr[3] = (byte) (i < 1000 ? 15 : PsExtractor.VIDEO_STREAM_MASK);
        bArr[4] = b;
        bArr[5] = b2;
        bArr[6] = (byte) i2;
        int checksum = checksum(bArr);
        bArr[7] = (byte) ((checksum >> 8) & 255);
        bArr[8] = (byte) (checksum & 255);
        return bArr;
    }

    private void getParameters(int i, int i2, int i3) {
        String str = TAG;
        DLog.e(str, "getParameters  paramAddress:" + i + "paramsNumber:" + i2 + "requestPriority:" + i3);
        enqueueRequest(new EcamRequest(2, getParametersPacket(i, i2)));
    }

    private boolean sendCommand(byte[] bArr) {
        DLog.e(TAG, "sendCommand");
        if (DeLonghiManager.getInstance().getCurrentSelectedEcam().getAppModelId().contains("striker")) {
            ((DeLonghiWifiConnectService) DeLonghi.getInstance().getConnectService()).getAylaNetworkInstance().setProperty(DeLonghiManager.getInstance().getCurrentSelectedEcam().getAylaDSN(), AylaProperties.DATA_REQUEST_STRIKER, Base64.encodeToString(bArr, 2), null);
            return true;
        } else if (this.mIsWifi) {
            ((DeLonghiWifiConnectService) DeLonghi.getInstance().getConnectService()).getAylaNetworkInstance().setProperty(DeLonghiManager.getInstance().getCurrentSelectedEcam().getAylaDSN(), AylaProperties.DATA_REQUEST, Base64.encodeToString(bArr, 2), null);
            return true;
        } else {
            enqueueRequest(new EcamRequest(2, bArr));
            return true;
        }
    }

    public void getProfilesNames(int i, int i2) {
        DLog.e(TAG, "getProfilesNames");
        int checksum = checksum(r0);
        byte[] bArr = {Ascii.CR, 7, PROFILES_NAME_READ_ANSWER_ID, -16, (byte) i, (byte) i2, (byte) ((checksum >> 8) & 255), (byte) (checksum & 255)};
        enqueueRequest(new EcamRequest(2, bArr));
    }

    public void getRecipesName(int i, int i2) {
        DLog.e(TAG, "getRecipesName");
        int checksum = checksum(r0);
        byte[] bArr = {Ascii.CR, 7, RECIPES_NAME_READ_ANSWER_ID, -16, (byte) i, (byte) i2, (byte) ((checksum >> 8) & 255), (byte) (checksum & 255)};
        enqueueRequest(new EcamRequest(2, bArr));
    }

    public void getBeanSystems(int i) {
        String str = TAG;
        DLog.e(str, "getBeanSystems index:" + i);
        int checksum = checksum(r0);
        byte[] bArr = {Ascii.CR, 6, BEAN_SYSTEM_READ_ANSWER_ID, -16, (byte) i, (byte) ((checksum >> 8) & 255), (byte) (checksum & 255)};
        enqueueRequest(new EcamRequest(2, bArr));
    }

    public void getBeanSystemPar(int i, int i2) {
        String str = TAG;
        DLog.e(str, "getBeanSystemPar paramAddress:" + i + "   paramsNumber:" + i2);
        getParameters(i, i2, 2);
    }

    public void getRecipesPriority(int i) {
        String str = TAG;
        DLog.e(str, "getRecipesPriority profileId:" + i);
        int checksum = checksum(r0);
        byte[] bArr = {Ascii.CR, 6, -88, -16, (byte) i, (byte) ((checksum >> 8) & 255), (byte) (checksum & 255)};
        enqueueRequest(new EcamRequest(1, bArr));
    }

    public void synchRecipeQty(int i) {
        String str = TAG;
        DLog.e(str, "synchRecipeQty beverageId:" + i);
        int checksum = checksum(r0);
        byte[] bArr = {Ascii.CR, 6, -80, -16, (byte) i, (byte) ((checksum >> 8) & 255), (byte) (checksum & 255)};
        enqueueRequest(new EcamRequest(2, bArr));
    }

    public void getRecipesQty(int i, int i2, int i3) {
        String str = TAG;
        Log.e(str, "getRecipesQty: " + i + " beverageId: " + i2 + " Payload: " + bytesToHexString(recipeQtyPacket(i, i2, i3)));
        enqueueRequest(new EcamRequest(2, recipeQtyPacket(i, i2, i3)));
    }

    public static byte[] recipeQtyPacket(int i, int i2, int i3) {
        String str = TAG;
        DLog.e(str, "recipeQtyPacket profileId:" + i + " beverageId:" + i2);
        int checksum = checksum(r5);
        byte[] bArr = {Ascii.CR, 7, -90, -16, (byte) i, (byte) i2, (byte) ((checksum >> 8) & 255), (byte) (checksum & 255)};
        return bArr;
    }

    public void getStatisticalParameters(int i, int i2) {
        String str = TAG;
        DLog.e(str, "getStatisticalParameters paramAddress:" + i + " paramsNumber:" + i2);
        getStatParameters(i, i2, 2);
    }

    public void getFlowTime(int i, int i2) {
        String str = TAG;
        DLog.e(str, "getFlowTime paramAddress:" + i + " paramsNumber:" + i2);
        getParameters(i, i2, 2);
    }

    public void profileSelection(int i) {
        String str = TAG;
        DLog.e(str, "profileSelection profileId:" + i);
        enqueueRequest(new EcamRequest(2, getPacketForSendProfile(i)));
    }

    public static byte[] getPacketForSendProfile(int i) {
        String str = TAG;
        DLog.e(str, "getPacketForSendProfile profileId:" + i);
        int checksum = checksum(r0);
        byte[] bArr = {Ascii.CR, 6, -87, -16, (byte) i, (byte) ((checksum >> 8) & 255), (byte) (checksum & 255)};
        return bArr;
    }

    public void setHour(int i, int i2) {
        String str = TAG;
        DLog.e(str, "setHour hour:" + i + " setMin min:" + i2);
        int checksum = checksum(r0);
        byte[] bArr = {Ascii.CR, 7, -30, -16, (byte) i, (byte) i2, (byte) ((checksum >> 8) & 255), (byte) (checksum & 255)};
        enqueueRequest(new EcamRequest(2, bArr));
    }

    public void setParameter(int i, int i2) {
        String str = TAG;
        DLog.e(str, "setParameter paramAddress:" + i + "   dataToWrite:" + i2);
        enqueueRequest(new EcamRequest(2, getPacketForWriteParameter(i, i2)));
    }

    public static byte[] getPacketForWriteParameter(int i, int i2) {
        String str = TAG;
        DLog.e(str, "getPacketForParameter paramAddress:" + i + "   dataToWrite:" + i2);
        byte[] bArr = new byte[12];
        byte b = (byte) (i >> 8);
        byte b2 = (byte) i;
        bArr[0] = Ascii.CR;
        bArr[1] = Ascii.VT;
        bArr[2] = -112;
        bArr[3] = (byte) (i < 1000 ? 15 : PsExtractor.VIDEO_STREAM_MASK);
        bArr[4] = b;
        bArr[5] = b2;
        bArr[6] = (byte) (i2 >> 24);
        bArr[7] = (byte) (i2 >> 16);
        bArr[8] = (byte) (i2 >> 8);
        bArr[9] = (byte) i2;
        int checksum = checksum(bArr);
        bArr[10] = (byte) ((checksum >> 8) & 255);
        bArr[11] = (byte) (checksum & 255);
        return bArr;
    }

    public static byte[] getPacketForRefreshAppId() {
        int checksum = checksum(r0);
        byte[] bArr = {Ascii.CR, 7, -124, Ascii.SI, 3, 2, (byte) ((checksum >> 8) & 255), (byte) (checksum & 255)};
        return bArr;
    }

    public static byte[] getPacketForReadParameter(int i, int i2) {
        String str = TAG;
        DLog.e(str, "getPacketForParameter paramAddress:" + i + "   dataToWrite:" + i2);
        byte[] bArr = new byte[12];
        byte b = (byte) (i >> 8);
        byte b2 = (byte) i;
        bArr[0] = Ascii.CR;
        bArr[1] = 8;
        bArr[2] = -107;
        bArr[3] = (byte) (i < 1000 ? 15 : PsExtractor.VIDEO_STREAM_MASK);
        bArr[4] = b;
        bArr[5] = b2;
        bArr[6] = (byte) i2;
        int checksum = checksum(bArr);
        bArr[7] = (byte) ((checksum >> 8) & 255);
        bArr[8] = (byte) (checksum & 255);
        return bArr;
    }

    public static byte[] getPacketForReadSettingsParameter(int i, int i2) {
        String str = TAG;
        DLog.e(str, "getPacketForReadSettingsParameter paramAddress:" + i + "   dataToWrite:" + i2);
        byte[] bArr = new byte[9];
        byte b = (byte) (i >> 8);
        byte b2 = (byte) i;
        bArr[0] = Ascii.CR;
        bArr[1] = 8;
        bArr[2] = -107;
        bArr[3] = (byte) (i < 1000 ? 15 : PsExtractor.VIDEO_STREAM_MASK);
        bArr[4] = b;
        bArr[5] = b2;
        bArr[6] = (byte) i2;
        int checksum = checksum(bArr);
        bArr[7] = (byte) ((checksum >> 8) & 255);
        bArr[8] = (byte) (checksum & 255);
        return bArr;
    }

    public void setPin(String str) {
        if (str.length() < 4) {
            DLog.e(TAG, "Pin too short!");
            return;
        }
        int checksum = checksum(r0);
        byte[] bArr = {Ascii.CR, 9, -79, -16, (byte) str.charAt(0), (byte) str.charAt(1), (byte) str.charAt(2), (byte) str.charAt(3), (byte) ((checksum >> 8) & 255), (byte) (checksum & 255)};
        enqueueRequest(new EcamRequest(2, bArr));
    }

    public void setPinActivation(boolean z) {
        int checksum = checksum(r0);
        byte[] bArr = {Ascii.CR, 6, -80, -16, z ? (byte) 1 : (byte) 0, (byte) ((checksum >> 8) & 255), (byte) (checksum & 255)};
        enqueueRequest(new EcamRequest(2, bArr));
    }

    public void setProfilesNames(int i, int i2, String[] strArr, int[] iArr) {
        DLog.e(TAG, "setProfilesNames");
        enqueueRequest(new EcamRequest(2, getPacketForSetProfileName(i, i2, strArr, iArr)));
    }

    public void setProfileNamesForStriker(int i, int i2, String[] strArr, int[] iArr, int[] iArr2) {
        enqueueRequest(new EcamRequest(2, getPacketForSetStrikerProfileName(i, i2, strArr, iArr, iArr2)));
    }

    public static byte[] getPacketForSetFavoriteBeverage(int i, byte[] bArr) {
        byte[] bArr2 = new byte[19];
        bArr2[0] = Ascii.CR;
        bArr2[1] = Ascii.DC2;
        bArr2[2] = -83;
        bArr2[3] = -16;
        bArr2[4] = (byte) i;
        System.arraycopy(bArr, 0, bArr2, 5, bArr.length);
        int checksum = checksum(bArr2);
        bArr2[17] = (byte) ((checksum >> 8) & 255);
        bArr2[18] = (byte) (checksum & 255);
        String str = TAG;
        Log.d(str, "getPacketForSetFavoriteBeverage -> " + Utils.byteArrayToHex(bArr2));
        return bArr2;
    }

    public static byte[] getPacketForSetProfileName(int i, int i2, String[] strArr, int[] iArr) {
        DLog.e(TAG, "getPacketForSetProfileName");
        int length = (strArr.length * 21) + 8;
        byte[] bArr = new byte[length];
        bArr[0] = Ascii.CR;
        int i3 = length - 1;
        bArr[1] = (byte) i3;
        bArr[2] = -91;
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
        int checksum = checksum(bArr);
        bArr[length - 2] = (byte) ((checksum >> 8) & 255);
        bArr[i3] = (byte) (checksum & 255);
        return bArr;
    }

    public static byte[] getPacketForSetStrikerProfileName(int i, int i2, String[] strArr, int[] iArr, int[] iArr2) {
        DLog.e(TAG, "getPacketForSetProfileName");
        int length = (strArr.length * 22) + 8;
        byte[] bArr = new byte[length];
        bArr[0] = Ascii.CR;
        int i3 = length - 1;
        bArr[1] = (byte) i3;
        bArr[2] = -91;
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
            int i7 = i4 + 1;
            bArr[i7] = (byte) iArr2[i5];
            i4 = i7 + 1;
        }
        int checksum = checksum(bArr);
        bArr[length - 2] = (byte) ((checksum >> 8) & 255);
        bArr[i3] = (byte) (checksum & 255);
        return bArr;
    }

    public void setRecipesName(int i, int i2, String[] strArr, int[] iArr) {
        DLog.e(TAG, "setRecipesName");
        enqueueRequest(new EcamRequest(2, getPacketForSaveRecipeName(i, i2, strArr, iArr)));
    }

    public static byte[] getPacketForSaveRecipeName(int i, int i2, String[] strArr, int[] iArr) {
        DLog.e(TAG, "getPacketForSaveRecipeName");
        int length = (strArr.length * 21) + 8;
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
        int checksum = checksum(bArr);
        bArr[length - 2] = (byte) ((checksum >> 8) & 255);
        bArr[i3] = (byte) (checksum & 255);
        return bArr;
    }

    public void saveBeanSystem(int i, int i2, String str, int i3, float f, int i4) {
        String str2 = TAG;
        DLog.i(str2, "setBeanSystem id:" + i + "   visible:" + i2 + "  name:" + str + "grinder:" + f + "    aroma:" + i4);
        enqueueRequest(new EcamRequest(1, getPacketForBeanSystemSaveOrDelete(i, i2, str, i3, f, i4)));
    }

    public static byte[] getPacketForBeanSystemSaveOrDelete(int i, int i2, String str, int i3, float f, int i4) {
        DLog.i(TAG, "getPacketForBeanSystemSave id:" + i + "   visible:" + i2 + "  name:" + str + " grinder:" + f + "    aroma:" + i4);
        if (((DeLonghiWifiConnectService) DeLonghi.getInstance().getConnectService()).isStriker()) {
            f *= 2.0f;
        }
        byte[] bArr = new byte[52];
        bArr[0] = Ascii.CR;
        bArr[1] = 51;
        bArr[2] = BEAN_SYSTEM_WRITE_ANSWER_ID;
        bArr[3] = -16;
        bArr[4] = (byte) i;
        int i5 = 5;
        byte[] stringToByteArray40 = Utils.stringToByteArray40(str);
        int i6 = 0;
        while (i6 < stringToByteArray40.length) {
            bArr[i5] = i6 < 40 ? stringToByteArray40[i6] : (byte) 0;
            i5++;
            i6++;
        }
        bArr[45] = (byte) f;
        bArr[46] = (byte) i3;
        bArr[47] = (byte) i4;
        bArr[49] = (byte) i2;
        int checksum = checksum(bArr);
        bArr[50] = (byte) ((checksum >> 8) & 255);
        bArr[51] = (byte) (checksum & 255);
        return bArr;
    }

    public void selectBeanSystem(int i) {
        String str = TAG;
        DLog.i(str, "selectBeanSystem id:" + i);
        enqueueRequest(new EcamRequest(1, getPacketForSelectBean(i)));
    }

    public static byte[] getPacketForSelectBean(int i) {
        String str = TAG;
        DLog.i(str, "getPacketForSelectBean id:" + i);
        int checksum = checksum(r0);
        byte[] bArr = {Ascii.CR, 6, BEAN_SYSTEM_SELECT_ANSWER_ID, -16, (byte) i, (byte) ((checksum >> 8) & 255), (byte) (checksum & 255)};
        return bArr;
    }

    public void turnOnMode() {
        DLog.i(TAG, "turnOnMode");
        enqueueRequest(new EcamRequest(2, getPacketForTurnOn()));
    }

    public static byte[] getPacketForTurnOn() {
        DLog.i(TAG, "getPacketForTurnOn");
        int checksum = checksum(r0);
        byte[] bArr = {Ascii.CR, 7, -124, Ascii.SI, 2, 1, (byte) ((checksum >> 8) & 255), (byte) (checksum & 255)};
        return bArr;
    }

    private void enqueueRequest(EcamRequest ecamRequest) {
        new EnqueueRequestTask(this).execute(ecamRequest);
    }

    /* JADX INFO: Access modifiers changed from: private */
    /* loaded from: classes2.dex */
    public class EnqueueRequestTask extends AsyncTask<EcamRequest, Void, Void> {
        private WeakReference<EcamManagerV2> mWeakReference;

        EnqueueRequestTask(EcamManagerV2 ecamManagerV2) {
            this.mWeakReference = new WeakReference<>(ecamManagerV2);
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

    /* JADX INFO: Access modifiers changed from: private */
    public void enqueueInBackground(EcamRequest ecamRequest) {
        synchronized (this.mRequestQueue) {
            this.mRequestQueue.add(ecamRequest);
            this.mRequestQueue.notify();
        }
    }

    /* loaded from: classes2.dex */
    private class EcamGattCallback extends BluetoothGattCallback {
        private final String TAG = EcamGattCallback.class.getName();
        private WeakReference<EcamManagerV2> mWeakReference;

        EcamGattCallback(EcamManagerV2 ecamManagerV2) {
            this.mWeakReference = new WeakReference<>(ecamManagerV2);
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
                byte[] bArr = EcamManagerV2.this.response;
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
            if (EcamManagerV2.this.isResponseComplete()) {
                this.mWeakReference.get().updateReceived(EcamManagerV2.this.response);
            }
        }

        @Override // android.bluetooth.BluetoothGattCallback
        public void onCharacteristicRead(BluetoothGatt bluetoothGatt, BluetoothGattCharacteristic bluetoothGattCharacteristic, int i) {
            String str = this.TAG;
            DLog.i(str, "onCharacteristicRead " + i);
        }

        @Override // android.bluetooth.BluetoothGattCallback
        public void onCharacteristicWrite(BluetoothGatt bluetoothGatt, BluetoothGattCharacteristic bluetoothGattCharacteristic, int i) {
            if (bluetoothGattCharacteristic.getUuid().toString().equalsIgnoreCase(Constants.TRANSFER_CHARACTERISTIC_UUID)) {
                DLog.d(this.TAG, Utils.byteArrayToHex(bluetoothGattCharacteristic.getValue()));
                if (EcamManagerV2.this.chunk != null) {
                    EcamManagerV2.access$1408(EcamManagerV2.this);
                    if (EcamManagerV2.this.nextChunkIdx >= EcamManagerV2.this.chunk.length) {
                        EcamManagerV2.this.chunk = null;
                        EcamManagerV2.this.mHandler.postDelayed(EcamManagerV2.this.mTimeout, 1000L);
                        return;
                    }
                    EcamManagerV2 ecamManagerV2 = EcamManagerV2.this;
                    ecamManagerV2.writeBytes(ecamManagerV2.chunk[EcamManagerV2.this.nextChunkIdx]);
                }
            }
        }

        @Override // android.bluetooth.BluetoothGattCallback
        public void onConnectionStateChange(BluetoothGatt bluetoothGatt, int i, int i2) {
            if (this.mWeakReference.get() != null) {
                if (i != 0) {
                    this.mWeakReference.get().mConnectedEcamMachineAddress = null;
                    this.mWeakReference.get().mUpdatesListener.onMachineDisconnected(bluetoothGatt.getDevice().getAddress());
                    bluetoothGatt.close();
                } else if (i2 == 2) {
                    DLog.d(this.TAG, "Connected to GATT server.");
                    String address = bluetoothGatt.getDevice().getAddress();
                    DeLonghiManager.getInstance().setCurrentEcamMachineAddress(address);
                    this.mWeakReference.get().mConnectedEcamMachineAddress = address;
                    DeLonghiManager.getInstance().CONNECTION_TYPE = DeLonghiManager.getInstance().CONNECTION_BLE;
                    bluetoothGatt.discoverServices();
                } else if (i2 == 0) {
                    DLog.d(this.TAG, "Disconnected from GATT server.");
                    DeLonghiManager.getInstance().setCurrentEcamMachineAddress(EcamManagerV2.this.mConnectedEcamMachineAddress);
                    this.mWeakReference.get().mConnectedEcamMachineAddress = null;
                    DeLonghiManager.getInstance().CONNECTION_TYPE = "";
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
                    BluetoothGattService service = bluetoothGatt.getService(EcamManagerV2.mEcamServiceUUIDs[0]);
                    if (service != null) {
                        this.mWeakReference.get().mEcamCharacteristic = service.getCharacteristic(EcamManagerV2.mEcamCharacteristicUUID);
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
                EcamManagerV2.this.disconnectFromCurrentEcamMachine();
            }
        }

        @Override // android.bluetooth.BluetoothGattCallback
        public void onDescriptorRead(BluetoothGatt bluetoothGatt, BluetoothGattDescriptor bluetoothGattDescriptor, int i) {
            String str = this.TAG;
            DLog.d(str, "onDescriptorRead " + i);
        }
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

    public void disconnectFromCurrentEcamMachine() {
        String str = this.mConnectedEcamMachineAddress;
        if (str == null) {
            DLog.w(TAG, "No machines connected.");
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
            DLog.w(TAG, "No machines connected.");
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

    /* JADX INFO: Access modifiers changed from: private */
    public boolean isResponseComplete() {
        byte[] bArr = this.response;
        return bArr != null && bArr.length >= 2 && Utils.byteToInt(bArr[1]) == this.response.length - 1;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public void updateReceived(byte[] bArr) {
        new UpdateReceivedTask(this).execute(bArr);
    }

    /* JADX INFO: Access modifiers changed from: private */
    /* loaded from: classes2.dex */
    public class UpdateReceivedTask extends AsyncTask<byte[], Void, Void> {
        private WeakReference<EcamManagerV2> mWeakReference;

        UpdateReceivedTask(EcamManagerV2 ecamManagerV2) {
            this.mWeakReference = new WeakReference<>(ecamManagerV2);
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

    public String getConnectedEcamMachineAddress() {
        return this.mConnectedEcamMachineAddress;
    }

    public boolean isBleActive() {
        return this.mBleManager.isBtEnabled();
    }

    public String getmConnectedEcamMachineAddress() {
        return this.mConnectedEcamMachineAddress;
    }

    public void setmConnectedEcamMachineAddress(String str) {
        this.mConnectedEcamMachineAddress = str;
    }

    public boolean ismIsWifi() {
        return this.mIsWifi;
    }

    public void setmIsWifi(boolean z) {
        this.mIsWifi = z;
    }
}