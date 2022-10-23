package it.delonghi;

import android.app.Activity;
import android.bluetooth.BluetoothDevice;
import android.content.Intent;
import android.os.AsyncTask;
import android.os.Bundle;
import android.os.Handler;
import android.os.IBinder;
import android.util.SparseArray;
import androidx.localbroadcastmanager.content.LocalBroadcastManager;
import it.delonghi.IECamService;
import it.delonghi.bluetooth.BleManager;
import it.delonghi.database.DatabaseAdapter;
import it.delonghi.database.UpdateCustomRecipeTask;
import it.delonghi.database.UpdateRecipeDataTask;
import it.delonghi.ecam.EcamManager;
import it.delonghi.ecam.EcamManagerV2;
import it.delonghi.ecam.EcamUtils;
import it.delonghi.ecam.itf.EcamUpdatesReceived;
import it.delonghi.ecam.model.EcamMachine;
import it.delonghi.ecam.model.MonitorData;
import it.delonghi.ecam.model.Parameter;
import it.delonghi.ecam.model.Profile;
import it.delonghi.ecam.model.RecipeData;
import it.delonghi.ecam.model.enums.BeverageId;
import it.delonghi.ecam.model.enums.BeverageTasteType;
import it.delonghi.ecam.model.enums.BeverageTasteValue;
import it.delonghi.ecam.model.enums.OperationTriggerId;
import it.delonghi.model.BeanSystem;
import it.delonghi.model.DefaultsTable;
import it.delonghi.model.MachineDefaults;
import it.delonghi.model.ParameterModel;
import it.delonghi.model.RecipeDefaults;
import it.delonghi.striker.events.ChecksumKoEvent;
import it.delonghi.striker.events.ChecksumOkEvent;
import it.delonghi.striker.events.MachineConnectedEvent;
import it.delonghi.striker.events.MachineDisconnectEvent;
import it.delonghi.striker.events.MachineTimeoutEvent;
import it.delonghi.striker.events.MonitorData0Event;
import it.delonghi.striker.events.MonitorData1Event;
import it.delonghi.striker.events.MonitorData2Event;
import it.delonghi.striker.events.ParameterReceivedEvent;
import it.delonghi.striker.events.ParameterWriteEvent;
import it.delonghi.striker.events.ProfileSelectedEvent;
import it.delonghi.striker.events.ProfilesNamesReceivedEvent;
import it.delonghi.striker.events.RecipesNamesReceivedEvent;
import it.delonghi.striker.events.RecipesNamesWritedEvent;
import it.delonghi.striker.events.RecipesPrioritiesEvent;
import it.delonghi.striker.events.RecipesQtyReceivedEvent;
import it.delonghi.striker.events.ScanBleFinishedEvent;
import it.delonghi.striker.events.SetMachineTimeEvent;
import it.delonghi.utils.DLog;
import it.delonghi.utils.Utils;
import it.delonghi.utils.comparators.RecipePriorityComparator;
import java.io.FileDescriptor;
import java.io.PrintWriter;
import java.lang.ref.WeakReference;
import java.sql.SQLException;
import java.util.ArrayList;
import java.util.Collection;
import java.util.HashMap;
import java.util.Iterator;
import java.util.Timer;
import java.util.TimerTask;
import org.greenrobot.eventbus.EventBus;

/* loaded from: classes2.dex */
public class EcamService extends IECamService {
    public static final int BEAN_SYSTEM_RECIPES_NUMBER = 6;
    public static final int CUSTOM_RECIPES_NUMBER = 6;
    public static final int DEFAULT_BEVERAGES_NUMBER = 18;
    private static final String TAG = EcamService.class.getName();
    public static final int TOTAL_RECIPES_NUMBER = 24;
    private final boolean mAllowRebind = true;
    private IBinder mBinder = new IECamService.EcamBinder();
    public EcamManager mEcamManager;
    public EcamManagerV2 mEcamManagerV2;
    private EcamUpdatesListener mListener;
    private LocalBroadcastManager mLocalBroadcastManager;
    private Timer mTimer;

    @Override // it.delonghi.IECamService
    public boolean connectToWifiMachine(String str) {
        return false;
    }

    @Override // it.delonghi.IECamService
    public void dispenseTestRecipe() {
    }

    @Override // it.delonghi.IECamService
    public ArrayList<RecipeData> getBeansystemRecipes() {
        return null;
    }

    @Override // it.delonghi.IECamService
    public int getProtocolVersion() {
        return 1;
    }

    @Override // it.delonghi.IECamService
    public void readBeanSystems(int i) {
    }

    @Override // it.delonghi.IECamService
    public void readFlowTime(int i, int i2) {
    }

    @Override // it.delonghi.IECamService
    public void readRecipeMinMax() {
    }

    @Override // it.delonghi.IECamService
    public void saveBeanSystem(BeanSystem beanSystem) {
    }

    @Override // it.delonghi.IECamService
    public void selectBeanSystem(int i) {
    }

    /* loaded from: classes2.dex */
    private class EcamUpdatesListener implements EcamUpdatesReceived {
        private WeakReference<EcamService> mEcamService;

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void beverageSavingResult(boolean z, boolean z2) {
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onBeanSystemReceived(BeanSystem beanSystem) {
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onBeanSystemWritten(boolean z) {
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onProfilesRecipeQuantitiesReceived(int i, RecipeData recipeData) {
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onRecipeQuantitiesReceived(int i, RecipeDefaults recipeDefaults) {
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onStrikerProfilesNamesReceived(ArrayList<String> arrayList, ArrayList<Integer> arrayList2, ArrayList<Integer> arrayList3) {
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onUpdateReceived(byte[] bArr) {
        }

        public EcamUpdatesListener(EcamService ecamService) {
            this.mEcamService = new WeakReference<>(ecamService);
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onMachineFound(String str, String str2) {
            DLog.d(EcamService.TAG, "New EcamMachine: " + str + ", " + str2);
            BluetoothDevice ecamDevice = EcamService.this.mEcamManager.getEcamDevice(str);
            String retrieveSkuFromName = EcamUtils.retrieveSkuFromName(str2);
            DLog.d(EcamService.TAG, "Sku: " + retrieveSkuFromName);
            String[] strArr = Constants.SKUS_TO_FILTER;
            int length = strArr.length;
            boolean z = false;
            int i = 0;
            while (true) {
                if (i >= length) {
                    break;
                } else if (strArr[i].equals(retrieveSkuFromName)) {
                    z = true;
                    break;
                } else {
                    i++;
                }
            }
            if (z) {
                return;
            }
            EcamMachine ecamMachine = new EcamMachine(ecamDevice.getAddress(), str2, 1, 6);
            if (this.mEcamService.get() != null) {
                DeLonghiManager.getInstance().addEcamMachine(ecamMachine);
            }
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onMonitorDataReceived(MonitorData monitorData) {
            int type = monitorData.getType();
            String str = EcamService.TAG;
            DLog.d(str, "onMonitorDataReceived: " + type);
            if (this.mEcamService.get() != null) {
                Bundle bundle = new Bundle();
                bundle.putParcelable(Constants.MONITOR_DATA_EXTRA, monitorData);
                if (EcamService.this.getConnectedEcam() != null) {
                    bundle.putParcelable(Constants.LAST_MONITOR_DATA_EXTRA, EcamService.this.getConnectedEcam().getLastData2());
                }
                if (type == 0) {
                    EventBus.getDefault().post(new MonitorData0Event(bundle));
                } else if (type == 1) {
                    EventBus.getDefault().post(new MonitorData1Event(bundle));
                } else {
                    EventBus.getDefault().post(new MonitorData2Event(bundle));
                }
                new StoreLastMonitorData(this.mEcamService.get()).execute(monitorData);
            }
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void scanBlufi() {
            if (this.mEcamService.get() != null) {
                new RetrieveEcamInfosFromDb(this.mEcamService.get()).execute(new Void[0]);
            }
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onMachineConnected(String str) {
            if (this.mEcamService.get() != null) {
                new InitMachineInfos(this.mEcamService.get()).execute(str);
            }
            DeLonghiManager.getInstance().setCurrentSelectedEcam(EcamService.this.getEcamMachineFromAddress(str));
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onMachineDisconnected(String str) {
            DeLonghiManager.getInstance().getCurrentSelectedEcam().setConnectionType(Constants.CONNECTION_TYPE_NON_CONNECTED);
            DeLonghiManager.getInstance().CONNECTION_TYPE = Constants.CONNECTION_TYPE_NON_CONNECTED;
            Bundle bundle = new Bundle();
            bundle.putString(Constants.ECAM_MACHINE_ADDRESS_EXTRA, str);
            EventBus.getDefault().post(new MachineDisconnectEvent(bundle));
            if (this.mEcamService.get() != null) {
                this.mEcamService.get().stopAlarmsBatch();
                this.mEcamService.get().disconnectFromCurrentMachine();
            }
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onProfileSelectionAnswer(int i, boolean z) {
            EcamMachine connectedEcam = EcamService.this.getConnectedEcam();
            if (z && connectedEcam != null) {
                connectedEcam.setSelectedProfileIndex(i);
            }
            Bundle bundle = new Bundle();
            bundle.putInt(Constants.PROFILE_ID_EXTRA, i);
            bundle.putBoolean(Constants.OPERATION_RESULT_EXTRA, z);
            EventBus.getDefault().post(new ProfileSelectedEvent(bundle));
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onRequestTimeout(int i) {
            Bundle bundle = new Bundle();
            bundle.putInt(Constants.REQUEST_ID_EXTRA, i);
            EventBus.getDefault().post(new MachineTimeoutEvent(bundle));
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onChecksumsReceived(short s, short s2, short[] sArr) {
            Bundle bundle = new Bundle();
            bundle.putShort(Constants.NAMES_CS_EXTRA, s);
            bundle.putShort(Constants.CUSTOM_RECIPES_CS_EXTRA, s2);
            bundle.putShortArray(Constants.RECIPES_QTY_CS_EXTRA, sArr);
            EventBus.getDefault().post(new ChecksumOkEvent(bundle));
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onProfilesNamesReceived(ArrayList<String> arrayList, ArrayList<Integer> arrayList2) {
            Bundle bundle = new Bundle();
            bundle.putStringArrayList(Constants.NAMES_EXTRA, arrayList);
            bundle.putIntegerArrayList(Constants.ICONS_EXTRA, arrayList2);
            EventBus.getDefault().post(new ProfilesNamesReceivedEvent(bundle));
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onRecipesNamesReceived(ArrayList<String> arrayList, ArrayList<Integer> arrayList2) {
            Bundle bundle = new Bundle();
            bundle.putStringArrayList(Constants.NAMES_EXTRA, arrayList);
            bundle.putIntegerArrayList(Constants.ICONS_EXTRA, arrayList2);
            EventBus.getDefault().post(new RecipesNamesReceivedEvent(bundle));
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onRecipesQuantityReceived(int i, ArrayList<RecipeData> arrayList) {
            Bundle bundle = new Bundle();
            bundle.putInt(Constants.PROFILE_ID_EXTRA, i);
            bundle.putParcelableArrayList(Constants.RECIPES_QTY_EXTRA, arrayList);
            EventBus.getDefault().post(new RecipesQtyReceivedEvent(bundle));
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onRequestChecksumKo(byte b) {
            Bundle bundle = new Bundle();
            bundle.putByte(Constants.REQUEST_ID_EXTRA, b);
            EventBus.getDefault().post(new ChecksumKoEvent(bundle));
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onParametersReceived(ArrayList<Parameter> arrayList) {
            Bundle bundle = new Bundle();
            bundle.putParcelableArrayList(Constants.PARAMETERS_EXTRA, arrayList);
            EventBus.getDefault().post(new ParameterReceivedEvent(bundle));
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onPrioritiesReceived(int i, int[] iArr) {
            Bundle bundle = new Bundle();
            bundle.putInt(Constants.PROFILE_ID_EXTRA, i);
            bundle.putIntArray(Constants.RECIPES_PRIORITIES_EXTRA, iArr);
            EventBus.getDefault().post(new RecipesPrioritiesEvent(bundle));
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onParameterWritten(int i, boolean z) {
            Bundle bundle = new Bundle();
            bundle.putInt(Constants.PARAMETER_ID_EXTRA, i);
            bundle.putBoolean(Constants.OPERATION_RESULT_EXTRA, z);
            EventBus.getDefault().post(new ParameterWriteEvent(bundle));
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onTimeSet(boolean z) {
            Bundle bundle = new Bundle();
            bundle.putBoolean(Constants.OPERATION_RESULT_EXTRA, z);
            EventBus.getDefault().post(new SetMachineTimeEvent(bundle));
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onRecipesNamesWritten(boolean z) {
            Bundle bundle = new Bundle();
            bundle.putBoolean(Constants.OPERATION_RESULT_EXTRA, z);
            EventBus.getDefault().post(new RecipesNamesWritedEvent(bundle));
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onProfilesNamesWritten(boolean z) {
            Bundle bundle = new Bundle();
            bundle.putBoolean(Constants.OPERATION_RESULT_EXTRA, z);
            EcamService.this.broadCastMessage(IECamService.ACTION_PROFILE_NAME_WRITE_RESULT, bundle);
        }
    }

    @Override // android.app.Service
    public void onCreate() {
        DLog.v(TAG, "onCreate");
        super.onCreate();
    }

    @Override // android.app.Service
    public void onDestroy() {
        DLog.v(TAG, "onDestroy Service");
        stopAlarmsBatch();
        this.mEcamManager.disconnectFromCurrentEcamMachine();
        super.onDestroy();
    }

    @Override // android.app.Service
    public IBinder onBind(Intent intent) {
        DLog.d(TAG, "onBind");
        this.mLocalBroadcastManager = LocalBroadcastManager.getInstance(this);
        this.mListener = new EcamUpdatesListener(this);
        this.mEcamManager = new EcamManager(getApplicationContext(), this.mListener);
        this.mEcamManagerV2 = new EcamManagerV2(getApplicationContext(), this.mListener);
        this.mDbAdapter = DatabaseAdapter.getInstance(getApplicationContext());
        this.mDefaultsTable = DefaultsTable.getInstance(getApplicationContext());
        initOfflineEcam();
        return this.mBinder;
    }

    @Override // android.app.Service
    public boolean onUnbind(Intent intent) {
        DLog.d(TAG, "onUnbind");
        getClass();
        return true;
    }

    @Override // android.app.Service, android.content.ComponentCallbacks
    public void onLowMemory() {
        DLog.v(TAG, "#ECAMSERVICE# onLowMemory Service");
        super.onLowMemory();
    }

    @Override // android.app.Service
    protected void dump(FileDescriptor fileDescriptor, PrintWriter printWriter, String[] strArr) {
        DLog.d(TAG, "#ECAMSERVICE# DUMP");
        super.dump(fileDescriptor, printWriter, strArr);
    }

    @Override // android.app.Service, android.content.ComponentCallbacks2
    public void onTrimMemory(int i) {
        String str = TAG;
        DLog.d(str, "#ECAMSERVICE# onTrimMemory " + i);
    }

    @Override // android.app.Service
    public void onTaskRemoved(Intent intent) {
        DLog.d(TAG, "onTaskRemoved");
    }

    public EcamManager getEcamManager() {
        return this.mEcamManager;
    }

    @Override // it.delonghi.IECamService
    public boolean isBleSupported() {
        return BleManager.isBleAvailable(getApplicationContext());
    }

    @Override // it.delonghi.IECamService
    public boolean isBleActive() {
        return this.mEcamManager.isBleActive();
    }

    @Override // it.delonghi.IECamService
    public void useConnectionManager(Activity activity) {
        this.mEcamManager.useConnectionManager(activity);
    }

    @Override // it.delonghi.IECamService
    public void updateRecipeData(RecipeData recipeData) {
        EcamMachine ecamMachine = DeLonghi.getInstance().getConnectService().ecamMachine();
        if (ecamMachine == null) {
            ecamMachine = this.mOfflineEcam;
        }
        int id = recipeData.getId();
        ecamMachine.resetNamesChecksum();
        ecamMachine.resetCustomRecipesQtyChecksum();
        if (id <= 18) {
            ecamMachine.getSelectedProfile().addRecipeData(recipeData);
            new UpdateRecipeDataTask(DeLonghi.getInstance().getApplicationContext(), ecamMachine).execute(recipeData);
            return;
        }
        ecamMachine.getCustomRecipes().put(id, recipeData);
        ecamMachine.resetNamesChecksum();
        ecamMachine.resetCustomRecipesQtyChecksum();
        new UpdateCustomRecipeTask(DeLonghi.getInstance().getApplicationContext()).execute(DeLonghi.getInstance().getConnectService());
    }

    @Override // it.delonghi.IECamService
    public void startEcamScan() {
        DeLonghiManager.getInstance().getEcamMachines().clear();
        this.mEcamManager.startEcamScan();
    }

    @Override // it.delonghi.IECamService
    public void stopEcamScan() {
        this.mEcamManager.stopEcamScan();
    }

    @Override // it.delonghi.IECamService
    public boolean isScanning() {
        return this.mEcamManager.isScanning();
    }

    @Override // it.delonghi.IECamService
    public boolean isManualDisconnect() {
        return this.mEcamManager.isManualDisconnect();
    }

    @Override // it.delonghi.IECamService
    public boolean connectToMachine(String str) {
        return this.mEcamManager.connectToEcamMachine(str);
    }

    @Override // it.delonghi.IECamService
    public void disconnectFromCurrentMachine() {
        this.mEcamManager.disconnectFromCurrentEcamMachine();
    }

    @Override // it.delonghi.IECamService
    public void silentDisconnectFromCurrentMachine() {
        this.mEcamManager.silentDisconnectFromCurrentEcamMachine();
    }

    @Override // it.delonghi.IECamService
    public void readChecksums() {
        this.mEcamManager.checksumVerification();
    }

    @Override // it.delonghi.IECamService
    public void readProfileNames(int i, int i2) {
        this.mEcamManager.getProfilesNames(i, i2);
    }

    @Override // it.delonghi.IECamService
    public void readCustomRecipesNames(int i, int i2) {
        this.mEcamManager.getRecipesName(i, i2);
    }

    @Override // it.delonghi.IECamService
    public void readRecipesQty(int i, boolean z) {
        this.mEcamManager.getRecipesQty(i, 1, z ? 24 : 18);
    }

    @Override // it.delonghi.IECamService
    public void readRecipeQty(int i, int i2) {
        this.mEcamManager.getRecipesQty(i, i2, i2);
    }

    @Override // it.delonghi.IECamService
    public void readParameters(int i, int i2) {
        this.mEcamManager.getParameters(i, i2);
    }

    @Override // it.delonghi.IECamService
    public void readStatisticalParameters(int i, int i2) {
        this.mEcamManager.getStatisticalParameters(i, i2);
    }

    @Override // it.delonghi.IECamService
    public void writeParameter(int i, int i2) {
        this.mEcamManager.setParameter(i, i2);
    }

    @Override // it.delonghi.IECamService
    public void setHour(int i, int i2) {
        this.mEcamManager.setHour(i, i2);
    }

    @Override // it.delonghi.IECamService
    public void saveRecipeData(int i, int i2, int i3, ArrayList<ParameterModel> arrayList, BeverageTasteValue beverageTasteValue, boolean z) {
        BeverageTasteType beverageTasteType = z ? BeverageTasteType.SAVE_BEVERAGE_INVERSION : BeverageTasteType.SAVE_BEVERAGE;
        if (beverageTasteValue == BeverageTasteValue.DELETE) {
            beverageTasteValue = BeverageTasteValue.PREGROUND;
        }
        this.mEcamManager.dispenseBeverage(i, OperationTriggerId.DONTCARE, i2, i3, beverageTasteValue, beverageTasteType);
    }

    @Override // it.delonghi.IECamService
    public void saveRecipeData(int i, int i2, int i3, int i4, ArrayList<ParameterModel> arrayList, BeverageTasteValue beverageTasteValue, boolean z) {
        BeverageTasteType beverageTasteType = z ? BeverageTasteType.SAVE_BEVERAGE_INVERSION : BeverageTasteType.SAVE_BEVERAGE;
        if (beverageTasteValue == BeverageTasteValue.DELETE) {
            beverageTasteValue = BeverageTasteValue.PREGROUND;
        }
        this.mEcamManager.dispenseBeverage(i, OperationTriggerId.DONTCARE, i3, i4, beverageTasteValue, beverageTasteType);
    }

    @Override // it.delonghi.IECamService
    public void saveRecipeName(int i, String str, int i2) {
        int i3 = i - 18;
        this.mEcamManager.setRecipesName(i3, i3, new String[]{str}, new int[]{i2});
    }

    @Override // it.delonghi.IECamService
    public void saveProfileName(int i, String str, int i2) {
        this.mEcamManager.setProfilesNames(i, i, new String[]{str}, new int[]{i2});
    }

    @Override // it.delonghi.IECamService
    public void deleteRecipe(int i, ArrayList<ParameterModel> arrayList) {
        saveRecipeData(i, 0, 0, null, BeverageTasteValue.DELETE, false);
    }

    @Override // it.delonghi.IECamService
    public void dispenseRecipe(int i, int i2, int i3, ArrayList<ParameterModel> arrayList, BeverageTasteValue beverageTasteValue, boolean z) {
        this.mEcamManager.dispenseBeverage(i, OperationTriggerId.START, i2, i3, beverageTasteValue, z ? BeverageTasteType.PREPARE_BEVERAGE_INVERSION : BeverageTasteType.PREPARE_BEVERAGE);
    }

    @Override // it.delonghi.IECamService
    public void stopRecipeDispensing(int i, int i2, int i3, ArrayList<ParameterModel> arrayList, BeverageTasteValue beverageTasteValue, boolean z) {
        this.mEcamManager.dispenseBeverage(i, OperationTriggerId.STOP, i2, i3, beverageTasteValue, z ? BeverageTasteType.PREPARE_BEVERAGE_INVERSION : BeverageTasteType.PREPARE_BEVERAGE);
    }

    @Override // it.delonghi.IECamService
    public void readRecipesPriorities(int i) {
        this.mEcamManager.getRecipesPriority(i);
    }

    @Override // it.delonghi.IECamService
    public void turnOnMode() {
        this.mEcamManager.turnOnMode();
    }

    @Override // it.delonghi.IECamService
    public void readData2() {
        this.mEcamManager.getMonitorMode(2);
    }

    @Override // it.delonghi.IECamService
    public EcamMachine getDemoEcam() {
        if (DeLonghiManager.getInstance().getCurrentSelectedEcam() == null) {
            DeLonghiManager.getInstance().setCurrentSelectedEcam(this.mOfflineEcam);
        }
        return this.mOfflineEcam;
    }

    @Override // it.delonghi.IECamService
    public EcamMachine getConnectedEcam() {
        return getEcamMachineFromAddress(this.mEcamManager.getConnectedEcamMachineAddress());
    }

    @Override // it.delonghi.IECamService
    public EcamMachine getEcamMachineFromAddress(String str) {
        if (str == null || !DeLonghiManager.getInstance().getEcamMachines().containsKey(str)) {
            return null;
        }
        return DeLonghiManager.getInstance().getEcamMachines().get(str);
    }

    @Override // it.delonghi.IECamService
    public ArrayList<EcamMachine> getScannedEcamMachines() {
        if (DeLonghiManager.getInstance().getEcamMachines() != null) {
            return new ArrayList<>(DeLonghiManager.getInstance().getEcamMachines().values());
        }
        return new ArrayList<>();
    }

    @Override // it.delonghi.IECamService
    public ArrayList<EcamMachine> getScannedEcamMachinesNotConnected() {
        if (DeLonghiManager.getInstance().getEcamMachines() != null) {
            HashMap hashMap = new HashMap(DeLonghiManager.getInstance().getEcamMachines());
            hashMap.remove(this.mEcamManager.getConnectedEcamMachineAddress());
            return new ArrayList<>(hashMap.values());
        }
        return new ArrayList<>();
    }

    /* JADX INFO: Access modifiers changed from: private */
    public void broadCastMessage(String str, Bundle bundle) {
        String str2 = TAG;
        DLog.d(str2, "broadCastMessage: " + str);
        Intent intent = new Intent(str);
        if (bundle != null) {
            intent.putExtras(bundle);
        }
        this.mLocalBroadcastManager.sendBroadcast(intent);
    }

    private void startBatch(long j) {
        DLog.v(TAG, "startBatch");
        if (this.mTimer == null) {
            TimerTask timerTask = new TimerTask() { // from class: it.delonghi.EcamService.1
                private int cycles = 0;

                @Override // java.util.TimerTask, java.lang.Runnable
                public void run() {
                    int i = this.cycles;
                    if (i == 0 || i % 5 == 0) {
                        EcamService.this.mEcamManager.getMonitorMode(2);
                    }
                    this.cycles++;
                }
            };
            Timer timer = new Timer();
            this.mTimer = timer;
            timer.schedule(timerTask, 0L, j);
        }
        String str = TAG;
        DLog.d(str, "Task scheduled every " + j + "ms.");
    }

    private void stopBatch() {
        DLog.v(TAG, "stopBatch");
        Timer timer = this.mTimer;
        if (timer != null) {
            timer.cancel();
            this.mTimer = null;
        }
    }

    @Override // it.delonghi.IECamService
    public void startAlarmsBatch() {
        stopBatch();
        startBatch(1000L);
    }

    @Override // it.delonghi.IECamService
    public void stopAlarmsBatch() {
        stopBatch();
    }

    @Override // it.delonghi.IECamService
    public void startDispensingBatch() {
        stopBatch();
        startBatch(400L);
    }

    @Override // it.delonghi.IECamService
    public void stopDispensingBatch() {
        stopBatch();
    }

    @Override // it.delonghi.IECamService
    public void profileSelection(int i) {
        this.mEcamManager.profileSelection(i);
    }

    /* JADX INFO: Access modifiers changed from: private */
    public void onEcamInsertInDbFinished(String str) {
        new Handler().postDelayed(new Runnable() { // from class: it.delonghi.EcamService.2
            @Override // java.lang.Runnable
            public void run() {
                EcamService.this.startAlarmsBatch();
            }
        }, 500L);
        Bundle bundle = new Bundle();
        bundle.putString(Constants.ECAM_MACHINE_ADDRESS_EXTRA, str);
        EventBus.getDefault().post(new MachineConnectedEvent(bundle));
    }

    /* JADX INFO: Access modifiers changed from: private */
    public void storeCurrentMonitorData(MonitorData monitorData) {
        EcamMachine ecamMachine;
        String connectedEcamMachineAddress = this.mEcamManager.getConnectedEcamMachineAddress();
        if (connectedEcamMachineAddress == null || !DeLonghiManager.getInstance().getEcamMachines().containsKey(connectedEcamMachineAddress) || (ecamMachine = DeLonghiManager.getInstance().getEcamMachines().get(connectedEcamMachineAddress)) == null || monitorData.getType() != 2) {
            return;
        }
        ecamMachine.setLastData2(monitorData);
    }

    @Override // it.delonghi.IECamService
    public MonitorData getLastMonitorData(int i) {
        EcamMachine connectedEcam = getConnectedEcam();
        if (connectedEcam == null || i != 2) {
            return null;
        }
        return connectedEcam.getLastData2();
    }

    @Override // it.delonghi.IECamService
    public SparseArray<Profile> getProfiles() {
        EcamMachine connectedEcam = getConnectedEcam();
        if (connectedEcam != null) {
            return connectedEcam.getProfiles();
        }
        return this.mOfflineEcam.getProfiles();
    }

    @Override // it.delonghi.IECamService
    public int getSelectedProfileIndex() {
        EcamMachine connectedEcam = getConnectedEcam();
        if (connectedEcam != null) {
            return connectedEcam.getSelectedProfileIndex();
        }
        return this.mOfflineEcam.getSelectedProfileIndex();
    }

    @Override // it.delonghi.IECamService
    public void setSelectedProfileIndex(int i) {
        EcamMachine connectedEcam = getConnectedEcam();
        if (connectedEcam != null) {
            connectedEcam.setSelectedProfileIndex(i);
        }
    }

    public String getMachineName() {
        EcamMachine connectedEcam = getConnectedEcam();
        if (connectedEcam != null) {
            return EcamUtils.getMachineNameToDisplay(connectedEcam);
        }
        return EcamUtils.getMachineNameToDisplay(this.mOfflineEcam);
    }

    @Override // it.delonghi.IECamService
    public RecipeData getRecipeData(int i) {
        SparseArray<RecipeData> customRecipes;
        EcamMachine connectedEcam = getConnectedEcam();
        if (connectedEcam == null) {
            connectedEcam = this.mOfflineEcam;
        }
        if (i < BeverageId.CUSTOM_01.getValue()) {
            customRecipes = connectedEcam.getSelectedProfile().getRecipesData();
        } else {
            customRecipes = connectedEcam.getCustomRecipes();
        }
        if (customRecipes != null) {
            return customRecipes.get(i);
        }
        return null;
    }

    @Override // it.delonghi.IECamService
    public ArrayList<RecipeData> getLastPreparedBeverages() {
        if (getConnectedEcam() != null) {
            ArrayList<RecipeData> classicRecipes = getClassicRecipes();
            classicRecipes.addAll(getCustomRecipes());
            classicRecipes.sort(new RecipePriorityComparator());
            return classicRecipes;
        }
        return null;
    }

    @Override // it.delonghi.IECamService
    public ArrayList<RecipeData> getClassicRecipes() {
        EcamMachine connectedEcam = getConnectedEcam();
        if (connectedEcam == null) {
            connectedEcam = this.mOfflineEcam;
        }
        ArrayList asList = Utils.asList(connectedEcam.getSelectedProfile().getRecipesData());
        ArrayList<RecipeData> arrayList = new ArrayList<>();
        if (asList != null) {
            Iterator it2 = asList.iterator();
            while (it2.hasNext()) {
                RecipeData recipeData = (RecipeData) it2.next();
                RecipeDefaults defaultValuesForRecipe = this.mDefaultsTable.getDefaultValuesForRecipe(connectedEcam.getOriginalName(), recipeData.getId());
                if (defaultValuesForRecipe != null && defaultValuesForRecipe.isPresent() && recipeData.getPriority() != 255 && recipeData.getId() <= 18) {
                    arrayList.add(recipeData);
                }
            }
        }
        return arrayList;
    }

    @Override // it.delonghi.IECamService
    public ArrayList<RecipeData> getAllClassicRecipes() {
        EcamMachine connectedEcam = getConnectedEcam();
        if (connectedEcam == null) {
            connectedEcam = this.mOfflineEcam;
        }
        ArrayList<RecipeData> arrayList = new ArrayList<>();
        Iterator it2 = Utils.asList(connectedEcam.getSelectedProfile().getRecipesData()).iterator();
        while (it2.hasNext()) {
            RecipeData recipeData = (RecipeData) it2.next();
            if (this.mDefaultsTable.getDefaultValuesForRecipe(connectedEcam.getOriginalName(), recipeData.getId()).isPresent()) {
                arrayList.add(recipeData);
            }
        }
        return arrayList;
    }

    @Override // it.delonghi.IECamService
    public RecipeData getNextCustomRecipe() {
        EcamMachine connectedEcam = getConnectedEcam();
        if (connectedEcam == null) {
            connectedEcam = this.mOfflineEcam;
        }
        SparseArray<RecipeData> customRecipes = connectedEcam.getCustomRecipes();
        for (int value = BeverageId.CUSTOM_01.getValue(); value <= 24; value++) {
            RecipeData recipeData = customRecipes.get(value);
            if (recipeData.isCustomBeverage() && !recipeData.isCreated()) {
                return recipeData;
            }
        }
        return null;
    }

    /* loaded from: classes2.dex */
    private class RetrieveEcamInfosFromDb extends AsyncTask<Void, Integer, Void> {
        private WeakReference<EcamService> mReference;

        public RetrieveEcamInfosFromDb(EcamService ecamService) {
            this.mReference = new WeakReference<>(ecamService);
        }

        /* JADX INFO: Access modifiers changed from: protected */
        @Override // android.os.AsyncTask
        public Void doInBackground(Void... voidArr) {
            if (this.mReference.get() == null || DeLonghiManager.getInstance().getEcamMachines() == null) {
                return null;
            }
            DatabaseAdapter databaseAdapter = this.mReference.get().mDbAdapter;
            Collection<EcamMachine> values = DeLonghiManager.getInstance().getEcamMachines().values();
            try {
                databaseAdapter.open();
                for (EcamMachine ecamMachine : values) {
                    EcamMachine ecamMachine2 = databaseAdapter.getEcamMachine(ecamMachine.getAddress());
                    MachineDefaults defaultValuesForMachine = EcamService.this.mDefaultsTable.getDefaultValuesForMachine(ecamMachine.getOriginalName());
                    if (ecamMachine2 != null) {
                        ecamMachine2.setProductCode(defaultValuesForMachine.getProductCode());
                        ecamMachine2.setAppModelId(defaultValuesForMachine.getAppModelId());
                        ecamMachine2.setCustomRecipesCnt(defaultValuesForMachine.getnCustomRecipes());
                        ecamMachine2.setBeanSystemRecipesCnt(defaultValuesForMachine.getnBeanSystemRecipes());
                        ecamMachine2.setDefaultRecipesCnt(defaultValuesForMachine.getnStandardRecipes());
                        ecamMachine2.setProfileNumbers(defaultValuesForMachine.getnProfiles());
                        ecamMachine2.setConnectionType(ecamMachine.getConnectionType());
                        ecamMachine2.setProtocolVersion(defaultValuesForMachine.getProtocolVersion());
                        ecamMachine2.setProfileIconSet(defaultValuesForMachine.getProfileIconSet());
                        ecamMachine2.setProtocolMinorVersion(defaultValuesForMachine.getProtocolMinorVersion());
                        ecamMachine2.setGrindersCount(defaultValuesForMachine.getnGrinders());
                        DeLonghiManager.getInstance().addEcamMachine(ecamMachine2);
                    } else {
                        ecamMachine.setModelName(defaultValuesForMachine.getName());
                        ecamMachine.setAppModelId(defaultValuesForMachine.getAppModelId());
                        ecamMachine.setType(defaultValuesForMachine.getType());
                        ecamMachine.setProtocolVersion(defaultValuesForMachine.getProtocolVersion());
                        ecamMachine.setProfileNumbers(defaultValuesForMachine.getnProfiles());
                    }
                }
                databaseAdapter.close();
                return null;
            } catch (SQLException e) {
                e.printStackTrace();
                return null;
            }
        }

        /* JADX INFO: Access modifiers changed from: protected */
        @Override // android.os.AsyncTask
        public void onPostExecute(Void r2) {
            if (this.mReference.get() != null) {
                EventBus.getDefault().post(new ScanBleFinishedEvent());
            }
        }
    }

    /* loaded from: classes2.dex */
    private class InitMachineInfos extends AsyncTask<String, Integer, String> {
        private WeakReference<EcamService> mReference;

        public InitMachineInfos(EcamService ecamService) {
            this.mReference = new WeakReference<>(ecamService);
        }

        /* JADX INFO: Access modifiers changed from: protected */
        @Override // android.os.AsyncTask
        public String doInBackground(String... strArr) {
            EcamMachine ecamMachine;
            if (this.mReference.get() != null && (ecamMachine = DeLonghiManager.getInstance().getEcamMachines().get(strArr[0])) != null) {
                ecamMachine.setTemperature(null);
                DatabaseAdapter databaseAdapter = this.mReference.get().mDbAdapter;
                MachineDefaults defaultValuesForMachine = EcamService.this.mDefaultsTable.getDefaultValuesForMachine(ecamMachine.getOriginalName());
                try {
                    databaseAdapter.open();
                    EcamMachine ecamMachine2 = databaseAdapter.getEcamMachine(strArr[0]);
                    ecamMachine.setProfileNumbers(defaultValuesForMachine.getnProfiles());
                    if (ecamMachine2 == null) {
                        databaseAdapter.initEcamInDb(ecamMachine);
                    } else {
                        SparseArray<Profile> retrieveProfiles = databaseAdapter.retrieveProfiles(ecamMachine2.getAddress(), ecamMachine2.getProtocolVersion());
                        ecamMachine.setProfiles(retrieveProfiles);
                        SparseArray<RecipeData> retrieveCustomRecipesData = databaseAdapter.retrieveCustomRecipesData(ecamMachine2.getAddress(), 24);
                        if (retrieveCustomRecipesData != null) {
                            for (int i = 0; i < 6; i++) {
                                int value = BeverageId.CUSTOM_01.getValue() + i;
                                ecamMachine.getCustomRecipes().put(value, retrieveCustomRecipesData.get(value));
                            }
                        }
                        if (retrieveProfiles != null) {
                            for (int i2 = 1; i2 <= retrieveProfiles.size(); i2++) {
                                retrieveProfiles.get(i2).setRecipesData(databaseAdapter.retrieveRecipesData(ecamMachine.getAddress(), i2, 24));
                            }
                        }
                    }
                    databaseAdapter.close();
                } catch (SQLException e) {
                    e.printStackTrace();
                }
            }
            return strArr[0] != null ? strArr[0] : "";
        }

        /* JADX INFO: Access modifiers changed from: protected */
        @Override // android.os.AsyncTask
        public void onPostExecute(String str) {
            if (this.mReference.get() != null) {
                this.mReference.get().onEcamInsertInDbFinished(str);
            }
        }
    }

    /* loaded from: classes2.dex */
    private class StoreLastMonitorData extends AsyncTask<MonitorData, Void, Void> {
        private WeakReference<EcamService> mEcamService;

        public StoreLastMonitorData(EcamService ecamService) {
            this.mEcamService = new WeakReference<>(ecamService);
        }

        /* JADX INFO: Access modifiers changed from: protected */
        @Override // android.os.AsyncTask
        public Void doInBackground(MonitorData... monitorDataArr) {
            if (this.mEcamService.get() != null) {
                this.mEcamService.get().storeCurrentMonitorData(monitorDataArr[0]);
                return null;
            }
            return null;
        }
    }
}