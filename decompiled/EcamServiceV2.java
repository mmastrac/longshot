package it.delonghi;

import android.app.Activity;
import android.bluetooth.BluetoothDevice;
import android.content.Context;
import android.content.Intent;
import android.os.AsyncTask;
import android.os.Bundle;
import android.os.Handler;
import android.os.IBinder;
import android.util.Log;
import android.util.SparseArray;
import androidx.localbroadcastmanager.content.LocalBroadcastManager;
import it.delonghi.IECamService;
import it.delonghi.database.DatabaseAdapter;
import it.delonghi.database.UpdateCustomRecipeTask;
import it.delonghi.database.UpdateRecipeDataTask;
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
import it.delonghi.ecam.model.enums.IngredientsId;
import it.delonghi.ecam.model.enums.OperationTriggerId;
import it.delonghi.model.BeanSystem;
import it.delonghi.model.DefaultsTable;
import it.delonghi.model.MachineDefaults;
import it.delonghi.model.ParameterModel;
import it.delonghi.model.RecipeDefaults;
import it.delonghi.service.DeLonghiWifiConnectService;
import it.delonghi.striker.events.BeanSystemReceivedEvent;
import it.delonghi.striker.events.BeverageSaveResultEvent;
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
import it.delonghi.striker.events.RecipesDefQtyReceivedEvent;
import it.delonghi.striker.events.RecipesNamesReceivedEvent;
import it.delonghi.striker.events.RecipesNamesWritedEvent;
import it.delonghi.striker.events.RecipesPrioritiesEvent;
import it.delonghi.striker.events.RecipesQtyReceivedEvent;
import it.delonghi.striker.events.ScanBleFinishedEvent;
import it.delonghi.striker.events.SetMachineTimeEvent;
import it.delonghi.utils.DLog;
import it.delonghi.utils.Utils;
import it.delonghi.utils.comparators.RecipePriorityComparator;
import java.io.PrintStream;
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
public class EcamServiceV2 extends IECamService {
    public static final int BEAN_SYSTEM_RECIPES_NUMBER = 6;
    public static final int CUSTOM_BEVERAGES_ID_NAME = 230;
    public static final int CUSTOM_RECIPES_NUMBER = 6;
    public static final int DEFAULT_BEVERAGES_NUMBER = 18;
    private static final String TAG = EcamServiceV2.class.getName();
    public static final int TOTAL_RECIPES_NUMBER = 30;
    private EcamMachine ecamWifi;
    public EcamManagerV2 mEcamManager;
    private EcamUpdatesListener mListener;
    private LocalBroadcastManager mLocalBroadcastManager;
    private Timer mTimer;
    private ArrayList<RecipeData> tmpRecipeDatas;
    private SparseArray<RecipeDefaults> tmpSyncRecipeDatas;
    private int tmpSyncStopPar;
    private int tmpSyncIndx = -1;
    private IBinder mBinder = new IECamService.EcamBinder();
    private boolean mIsWifi = false;

    @Override // it.delonghi.IECamService
    public int getProtocolVersion() {
        return 2;
    }

    static /* synthetic */ int access$308(EcamServiceV2 ecamServiceV2) {
        int i = ecamServiceV2.tmpSyncIndx;
        ecamServiceV2.tmpSyncIndx = i + 1;
        return i;
    }

    /* JADX INFO: Access modifiers changed from: private */
    /* loaded from: classes2.dex */
    public class EcamUpdatesListener implements EcamUpdatesReceived {
        private WeakReference<EcamServiceV2> mEcamService;

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onUpdateReceived(byte[] bArr) {
        }

        EcamUpdatesListener(EcamServiceV2 ecamServiceV2) {
            this.mEcamService = new WeakReference<>(ecamServiceV2);
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onMachineFound(String str, String str2) {
            DLog.d(EcamServiceV2.TAG, "New EcamMachine: " + str + ", " + str2);
            BluetoothDevice ecamDevice = EcamServiceV2.this.mEcamManager.getEcamDevice(str);
            String retrieveSkuFromName = EcamUtils.retrieveSkuFromName(str2);
            DLog.d(EcamServiceV2.TAG, "Sku: " + retrieveSkuFromName);
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
            EcamMachine ecamMachine = new EcamMachine(ecamDevice.getAddress(), str2, 2, 6);
            if (this.mEcamService.get() != null) {
                DeLonghiManager.getInstance().addEcamMachine(ecamMachine);
            }
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onMonitorDataReceived(MonitorData monitorData) {
            int type = monitorData.getType();
            if (this.mEcamService.get() != null) {
                Bundle bundle = new Bundle();
                bundle.putParcelable(Constants.MONITOR_DATA_EXTRA, monitorData);
                if (EcamServiceV2.this.getConnectedEcam() != null) {
                    bundle.putParcelable(Constants.LAST_MONITOR_DATA_EXTRA, EcamServiceV2.this.getConnectedEcam().getLastData2());
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
                new InitMachineInfos(this.mEcamService.get(), EcamServiceV2.this.mDefaultsTable, null).execute(str);
            }
            if (EcamServiceV2.this.mIsWifi) {
                DeLonghiManager.getInstance().CONNECTION_TYPE = DeLonghiManager.getInstance().CONNECTION_WIFI;
                EcamServiceV2.this.mEcamManager.setmConnectedEcamMachineAddress(str);
            }
            DeLonghiManager.getInstance().setCurrentSelectedEcam(EcamServiceV2.this.getEcamMachineFromAddress(str));
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onMachineDisconnected(String str) {
            DeLonghiManager.getInstance().getCurrentSelectedEcam().setConnectionType(Constants.CONNECTION_TYPE_NON_CONNECTED);
            DeLonghiManager.getInstance().CONNECTION_TYPE = Constants.CONNECTION_TYPE_NON_CONNECTED;
            EcamServiceV2.this.stopAlarmsBatch();
            Bundle bundle = new Bundle();
            bundle.putString(Constants.ECAM_MACHINE_ADDRESS_EXTRA, str);
            EventBus.getDefault().post(new MachineDisconnectEvent(bundle));
            EcamServiceV2.this.disconnectFromCurrentMachine();
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onProfileSelectionAnswer(int i, boolean z) {
            Log.d("PROFILI", "on profile selection answer " + i);
            EcamMachine connectedEcam = EcamServiceV2.this.getConnectedEcam();
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
        public void onStrikerProfilesNamesReceived(ArrayList<String> arrayList, ArrayList<Integer> arrayList2, ArrayList<Integer> arrayList3) {
            Bundle bundle = new Bundle();
            bundle.putStringArrayList(Constants.NAMES_EXTRA, arrayList);
            bundle.putIntegerArrayList(Constants.ICONS_EXTRA, arrayList2);
            bundle.putIntegerArrayList(Constants.MUGS_EXTRA, arrayList2);
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
        public void onBeanSystemReceived(BeanSystem beanSystem) {
            Bundle bundle = new Bundle();
            if (beanSystem != null) {
                bundle.putParcelable(Constants.BEAN_SYSTEM_EXTRA, beanSystem);
            }
            EventBus.getDefault().post(new BeanSystemReceivedEvent(bundle));
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onTimeSet(boolean z) {
            Bundle bundle = new Bundle();
            bundle.putBoolean(Constants.OPERATION_RESULT_EXTRA, z);
            EventBus.getDefault().post(new SetMachineTimeEvent(bundle));
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onProfilesRecipeQuantitiesReceived(int i, RecipeData recipeData) {
            if (EcamServiceV2.this.tmpRecipeDatas.size() >= EcamServiceV2.this.tmpSyncIndx) {
                EcamServiceV2.this.tmpRecipeDatas.add(EcamServiceV2.this.tmpSyncIndx, recipeData);
                if (EcamServiceV2.this.tmpSyncIndx + 1 < EcamServiceV2.this.tmpSyncStopPar) {
                    EcamServiceV2.access$308(EcamServiceV2.this);
                    EcamServiceV2.this.mEcamManager.getRecipesQty(i, EcamServiceV2.this.mDefaultsTable.getDefaultValuesForMachine(EcamServiceV2.this.getConnectedEcam().getOriginalName()).getRecipeDefaults().valueAt(EcamServiceV2.this.tmpSyncIndx).getId(), -1);
                    return;
                }
                Bundle bundle = new Bundle();
                bundle.putInt(Constants.PROFILE_ID_EXTRA, i);
                bundle.putParcelableArrayList(Constants.RECIPES_QTY_EXTRA, EcamServiceV2.this.tmpRecipeDatas);
                EventBus.getDefault().post(new RecipesQtyReceivedEvent(bundle));
            }
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void beverageSavingResult(boolean z, boolean z2) {
            String str = EcamServiceV2.TAG;
            DLog.d(str, "beverageSavingResult : " + z);
            String str2 = EcamServiceV2.TAG;
            DLog.d(str2, "beveragePrepareResult : " + z2);
            if (this.mEcamService.get() != null) {
                Bundle bundle = new Bundle();
                bundle.putBoolean(Constants.OPERATION_RESULT_EXTRA, z);
                bundle.putBoolean(Constants.OPERATION_PREP_RESULT_EXTRA, z2);
                EventBus.getDefault().post(new BeverageSaveResultEvent(bundle));
            }
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
        public void onRecipesNamesWritten(boolean z) {
            Bundle bundle = new Bundle();
            bundle.putBoolean(Constants.OPERATION_RESULT_EXTRA, z);
            EventBus.getDefault().post(new RecipesNamesWritedEvent(bundle));
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onBeanSystemWritten(boolean z) {
            Bundle bundle = new Bundle();
            bundle.putBoolean(Constants.OPERATION_RESULT_EXTRA, z);
            EcamServiceV2.this.broadCastMessage(IECamService.ACTION_BEAN_SYSTEM_WRITE_RESULT, bundle);
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onProfilesNamesWritten(boolean z) {
            Bundle bundle = new Bundle();
            bundle.putBoolean(Constants.OPERATION_RESULT_EXTRA, z);
            EcamServiceV2.this.broadCastMessage(IECamService.ACTION_PROFILE_NAME_WRITE_RESULT, bundle);
        }

        @Override // it.delonghi.ecam.itf.EcamUpdatesReceived
        public void onRecipeQuantitiesReceived(int i, RecipeDefaults recipeDefaults) {
            EcamServiceV2.this.tmpSyncRecipeDatas.put(recipeDefaults.getId(), recipeDefaults);
            if (EcamServiceV2.this.tmpSyncIndx + 1 < EcamServiceV2.this.tmpSyncStopPar) {
                EcamServiceV2.access$308(EcamServiceV2.this);
                EcamServiceV2.this.mEcamManager.synchRecipeQty(EcamServiceV2.this.mDefaultsTable.getDefaultValuesForMachine(EcamServiceV2.this.getConnectedEcam().getOriginalName()).getRecipeDefaults().valueAt(EcamServiceV2.this.tmpSyncIndx).getId());
                return;
            }
            Bundle bundle = new Bundle();
            bundle.putSparseParcelableArray(Constants.RECIPES_QTY_EXTRA, EcamServiceV2.this.tmpSyncRecipeDatas);
            EventBus.getDefault().post(new RecipesDefQtyReceivedEvent(bundle));
        }
    }

    @Override // android.app.Service
    public void onCreate() {
        DLog.v(TAG, "onCreate");
        super.onCreate();
    }

    @Override // android.app.Service
    public void onDestroy() {
        DLog.v(TAG, "#ECAMSERVICE# onDestroy Service");
        stopAlarmsBatch();
        this.mEcamManager.disconnectFromCurrentEcamMachine();
        super.onDestroy();
    }

    @Override // android.app.Service, android.content.ComponentCallbacks
    public void onLowMemory() {
        DLog.v(TAG, "#ECAMSERVICE# onLowMemory Service");
        super.onLowMemory();
    }

    @Override // android.app.Service
    public IBinder onBind(Intent intent) {
        DLog.d(TAG, "onBind");
        this.mLocalBroadcastManager = LocalBroadcastManager.getInstance(this);
        this.mListener = new EcamUpdatesListener(this);
        this.mEcamManager = new EcamManagerV2(getApplicationContext(), this.mListener);
        this.mDbAdapter = DatabaseAdapter.getInstance(getApplicationContext());
        this.mDefaultsTable = DefaultsTable.getInstance(getApplicationContext());
        initOfflineEcamV2();
        return this.mBinder;
    }

    @Override // android.app.Service
    public boolean onUnbind(Intent intent) {
        DLog.d(TAG, "onUnbind");
        return true;
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

    public EcamManagerV2 getEcamManager() {
        return this.mEcamManager;
    }

    @Override // it.delonghi.IECamService
    public boolean isBleActive() {
        return this.mEcamManager.isBleActive();
    }

    @Override // it.delonghi.IECamService
    public void startEcamScan() {
        if (DeLonghiManager.getInstance().getEcamMachines() != null) {
            DeLonghiManager.getInstance().getEcamMachines().clear();
        }
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
    public void useConnectionManager(Activity activity) {
        this.mEcamManager.useConnectionManager(activity);
    }

    @Override // it.delonghi.IECamService
    public boolean connectToWifiMachine(String str) {
        this.mIsWifi = true;
        this.mListener.onMachineConnected(str);
        this.mEcamManager.setmIsWifi(true);
        return true;
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
    public void readBeanSystems(int i) {
        this.mEcamManager.getBeanSystems(i);
    }

    public void readBeanSystemPar(int i, int i2) {
        this.mEcamManager.getBeanSystemPar(i, i2);
    }

    @Override // it.delonghi.IECamService
    public void readRecipesQty(int i, boolean z) {
        MachineDefaults defaultValuesForMachine = this.mDefaultsTable.getDefaultValuesForMachine(getConnectedEcam().getOriginalName());
        this.tmpSyncStopPar = defaultValuesForMachine.getRecipeDefaults().size();
        this.tmpRecipeDatas = new ArrayList<>(defaultValuesForMachine.getRecipeDefaults().size());
        this.tmpSyncIndx = 0;
        this.mEcamManager.getRecipesQty(i, defaultValuesForMachine.getRecipeDefaults().valueAt(this.tmpSyncIndx).getId(), -1);
    }

    @Override // it.delonghi.IECamService
    public void readRecipeQty(int i, int i2) {
        this.tmpSyncStopPar = 0;
        this.tmpRecipeDatas = new ArrayList<>(1);
        this.tmpSyncIndx = 0;
        EcamManagerV2 ecamManagerV2 = this.mEcamManager;
        if (ecamManagerV2 != null) {
            ecamManagerV2.getRecipesQty(i, i2, -1);
        }
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
    public void readFlowTime(int i, int i2) {
        this.mEcamManager.getFlowTime(i, i2);
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
        if (beverageTasteValue != BeverageTasteValue.DELETE) {
            this.mEcamManager.dispenseBeverage(i, getSelectedProfileIndex(), OperationTriggerId.DONTCARE, arrayList, beverageTasteValue, BeverageTasteType.SAVE_BEVERAGE, false);
            return;
        }
        this.mEcamManager.dispenseBeverage(i, getSelectedProfileIndex(), OperationTriggerId.DONTCARE, arrayList, beverageTasteValue, BeverageTasteType.DELETE_BEVERAGE, false);
    }

    @Override // it.delonghi.IECamService
    public void saveRecipeData(int i, int i2, int i3, int i4, ArrayList<ParameterModel> arrayList, BeverageTasteValue beverageTasteValue, boolean z) {
        if (beverageTasteValue != BeverageTasteValue.DELETE) {
            this.mEcamManager.dispenseBeverage(i, i2, OperationTriggerId.DONTCARE, arrayList, beverageTasteValue, BeverageTasteType.SAVE_BEVERAGE, false);
            return;
        }
        this.mEcamManager.dispenseBeverage(i, i2, OperationTriggerId.DONTCARE, arrayList, beverageTasteValue, BeverageTasteType.DELETE_BEVERAGE, false);
    }

    @Override // it.delonghi.IECamService
    public void saveRecipeName(int i, String str, int i2) {
        int i3 = (i - 230) + 1;
        this.mEcamManager.setRecipesName(i3, i3, new String[]{str}, new int[]{i2});
    }

    @Override // it.delonghi.IECamService
    public void saveBeanSystem(BeanSystem beanSystem) {
        EcamManagerV2 ecamManagerV2 = this.mEcamManager;
        int id = beanSystem.getId();
        boolean isEnable = beanSystem.isEnable();
        ecamManagerV2.saveBeanSystem(id, isEnable ? 1 : 0, beanSystem.getName(), beanSystem.getTemperature(), beanSystem.getGrinder(), beanSystem.getAroma());
    }

    @Override // it.delonghi.IECamService
    public void selectBeanSystem(int i) {
        this.mEcamManager.selectBeanSystem(i);
    }

    @Override // it.delonghi.IECamService
    public void readRecipeMinMax() {
        MachineDefaults defaultValuesForMachine = this.mDefaultsTable.getDefaultValuesForMachine(getConnectedEcam().getOriginalName());
        this.tmpSyncStopPar = defaultValuesForMachine.getRecipeDefaults().size();
        this.tmpSyncRecipeDatas = new SparseArray<>();
        this.tmpSyncIndx = 0;
        this.mEcamManager.synchRecipeQty(defaultValuesForMachine.getRecipeDefaults().valueAt(this.tmpSyncIndx).getId());
    }

    @Override // it.delonghi.IECamService
    public void saveProfileName(int i, String str, int i2) {
        String[] strArr = {str};
        int[] iArr = {i2};
        EcamManagerV2 ecamManagerV2 = this.mEcamManager;
        if (ecamManagerV2 != null) {
            ecamManagerV2.setProfilesNames(i, i, strArr, iArr);
        }
    }

    public void saveStrikerProfileName(int i, String str, int i2, int i3) {
        this.mListener = new EcamUpdatesListener(this);
        EcamManagerV2 ecamManagerV2 = new EcamManagerV2(getApplicationContext(), this.mListener);
        this.mEcamManager = ecamManagerV2;
        ecamManagerV2.setProfileNamesForStriker(i, i, new String[]{str}, new int[]{i2}, new int[]{i3});
    }

    @Override // it.delonghi.IECamService
    public void deleteRecipe(int i, ArrayList<ParameterModel> arrayList) {
        saveRecipeData(i, 0, 0, arrayList, BeverageTasteValue.DELETE, false);
    }

    @Override // it.delonghi.IECamService
    public void dispenseRecipe(int i, int i2, int i3, ArrayList<ParameterModel> arrayList, BeverageTasteValue beverageTasteValue, boolean z) {
        BeverageTasteType beverageTasteType = BeverageTasteType.PREPARE_BEVERAGE;
        try {
            if (i == BeverageId.ESPRESSO_COFFEE_2X.getValue()) {
                ArrayList arrayList2 = new ArrayList();
                for (int i4 = 0; i4 < arrayList.size(); i4++) {
                    ParameterModel parameterModel = arrayList.get(i4);
                    if (parameterModel.getId() == IngredientsId.COFFEE.getValue()) {
                        parameterModel.setDefValue(parameterModel.getDefValue() / 2);
                    }
                    arrayList2.add(parameterModel);
                }
                arrayList.clear();
                arrayList.addAll(arrayList2);
            }
        } catch (Exception unused) {
        }
        this.mEcamManager.dispenseBeverage(i, getSelectedProfileIndex(), OperationTriggerId.START, arrayList, beverageTasteValue, beverageTasteType, false);
    }

    @Override // it.delonghi.IECamService
    public void stopRecipeDispensing(int i, int i2, int i3, ArrayList<ParameterModel> arrayList, BeverageTasteValue beverageTasteValue, boolean z) {
        this.mEcamManager.dispenseBeverage(i, getSelectedProfileIndex(), OperationTriggerId.STOPV2, null, beverageTasteValue, z ? BeverageTasteType.PREPARE_BEVERAGE_INVERSION : BeverageTasteType.PREPARE_BEVERAGE, false);
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
        if (DeLonghiManager.getInstance().getCurrentSelectedEcam() != null) {
            DeLonghiManager.getInstance().setCurrentSelectedEcam(this.mOfflineEcam);
        }
        return this.mOfflineEcam;
    }

    @Override // it.delonghi.IECamService
    public void dispenseTestRecipe() {
        this.mEcamManager.dispenseTEstBeverage();
    }

    @Override // it.delonghi.IECamService
    public EcamMachine getConnectedEcam() {
        EcamManagerV2 ecamManagerV2 = this.mEcamManager;
        if (ecamManagerV2 != null) {
            return getEcamMachineFromAddress(ecamManagerV2.getConnectedEcamMachineAddress());
        }
        return this.mOfflineEcam;
    }

    @Override // it.delonghi.IECamService
    public EcamMachine getEcamMachineFromAddress(String str) {
        if (str != null && DeLonghiManager.getInstance().getEcamMachines().containsKey(str)) {
            return DeLonghiManager.getInstance().getEcamMachines().get(str);
        }
        return this.mOfflineEcam;
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
            TimerTask timerTask = new TimerTask() { // from class: it.delonghi.EcamServiceV2.1
                private int cycles = 0;

                @Override // java.util.TimerTask, java.lang.Runnable
                public void run() {
                    int i = this.cycles;
                    if (i == 0 || i % 5 == 0) {
                        EcamServiceV2.this.mEcamManager.getMonitorMode(2);
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
    /* renamed from: startAlarmsBatch */
    public void lambda$onEcamInsertInDbFinished$0$EcamServiceV2() {
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
        new Handler().postDelayed(new Runnable() { // from class: it.delonghi.-$$Lambda$EcamServiceV2$gUVaaWIyJSUN25WFVLJB00xM-Sw
            @Override // java.lang.Runnable
            public final void run() {
                EcamServiceV2.this.lambda$onEcamInsertInDbFinished$0$EcamServiceV2();
            }
        }, 500L);
        Bundle bundle = new Bundle();
        bundle.putString(Constants.ECAM_MACHINE_ADDRESS_EXTRA, str);
        EventBus.getDefault().post(new MachineConnectedEvent(bundle));
    }

    public void storeCurrentMonitorData(MonitorData monitorData) {
        if (DeLonghiManager.getInstance().getCurrentSelectedEcam() != null) {
            EcamMachine currentSelectedEcam = DeLonghiManager.getInstance().getCurrentSelectedEcam();
            if (monitorData.getType() == 2) {
                currentSelectedEcam.setLastData2(monitorData);
            }
        }
    }

    @Override // it.delonghi.IECamService
    public MonitorData getLastMonitorData(int i) {
        EcamMachine ecamMachine = this.ecamWifi;
        if (ecamMachine == null) {
            ecamMachine = getConnectedEcam();
        }
        if (ecamMachine == null || i != 2) {
            return null;
        }
        return ecamMachine.getLastData2();
    }

    @Override // it.delonghi.IECamService
    public SparseArray<Profile> getProfiles() {
        EcamMachine connectedEcam = getConnectedEcam();
        if (connectedEcam != null) {
            return connectedEcam.getProfiles();
        }
        if (this.mOfflineEcam != null) {
            return this.mOfflineEcam.getProfiles();
        }
        return null;
    }

    @Override // it.delonghi.IECamService
    public int getSelectedProfileIndex() {
        EcamMachine connectedEcam = getConnectedEcam();
        if (connectedEcam != null) {
            return connectedEcam.getSelectedProfileIndex();
        }
        if (this.mOfflineEcam != null) {
            return this.mOfflineEcam.getSelectedProfileIndex();
        }
        return 1;
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
    public void updateRecipeData(RecipeData recipeData) {
        EcamMachine ecamMachine = DeLonghi.getInstance().getConnectService().ecamMachine();
        if (ecamMachine == null) {
            ecamMachine = this.mOfflineEcam;
        }
        int id = recipeData.getId();
        ecamMachine.resetNamesChecksum();
        ecamMachine.resetCustomRecipesQtyChecksum();
        if (id < 200) {
            Profile selectedProfile = ecamMachine.getSelectedProfile();
            selectedProfile.addRecipeData(recipeData);
            selectedProfile.resetRecipesQtyChecksum();
            new UpdateRecipeDataTask(DeLonghi.getInstance().getApplicationContext(), ecamMachine).execute(recipeData);
            return;
        }
        if (id < 229) {
            ecamMachine.getBeanSystemRecipes().put(id, recipeData);
        } else {
            ecamMachine.getCustomRecipes().put(id, recipeData);
        }
        ecamMachine.resetNamesChecksum();
        ecamMachine.resetCustomRecipesQtyChecksum();
        new UpdateCustomRecipeTask(DeLonghi.getInstance().getApplicationContext()).execute(DeLonghi.getInstance().getConnectService());
    }

    @Override // it.delonghi.IECamService
    public RecipeData getRecipeData(int i) {
        EcamMachine connectedEcam;
        SparseArray<RecipeData> customRecipes;
        if (this.ecamWifi != null) {
            connectedEcam = DeLonghi.getInstance().getConnectService().ecamMachine();
        } else {
            connectedEcam = getConnectedEcam();
        }
        if (connectedEcam == null) {
            connectedEcam = this.mOfflineEcam;
        }
        if (i < 200) {
            customRecipes = connectedEcam.getSelectedProfile().getRecipesData();
        } else if (i < 229) {
            customRecipes = connectedEcam.getBeanSystemRecipes();
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
        EcamMachine ecamMachine = this.ecamWifi;
        if (ecamMachine == null) {
            ecamMachine = getConnectedEcam();
        }
        if (ecamMachine != null) {
            ArrayList<RecipeData> classicRecipes = getClassicRecipes();
            ArrayList<RecipeData> customRecipesV2 = getCustomRecipesV2();
            PrintStream printStream = System.out;
            printStream.println("Bevande! : " + classicRecipes.size() + " altr " + customRecipesV2.size());
            classicRecipes.addAll(customRecipesV2);
            classicRecipes.sort(new RecipePriorityComparator());
            return classicRecipes;
        }
        return null;
    }

    @Override // it.delonghi.IECamService
    public ArrayList<RecipeData> getClassicRecipes() {
        EcamMachine ecamMachine = this.ecamWifi;
        if (ecamMachine == null) {
            ecamMachine = getConnectedEcam();
        }
        if (ecamMachine == null) {
            ecamMachine = this.mOfflineEcam;
        }
        if (ecamMachine == null) {
            return null;
        }
        ArrayList asList = Utils.asList(ecamMachine.getSelectedProfile().getRecipesData());
        ArrayList<RecipeData> arrayList = new ArrayList<>();
        if (asList != null) {
            Iterator it2 = asList.iterator();
            while (it2.hasNext()) {
                RecipeData recipeData = (RecipeData) it2.next();
                if (recipeData.getId() < 200) {
                    arrayList.add(recipeData);
                }
            }
        }
        return arrayList;
    }

    @Override // it.delonghi.IECamService
    public ArrayList<RecipeData> getBeansystemRecipes() {
        ArrayList arrayList = new ArrayList();
        EcamMachine ecamMachine = this.ecamWifi;
        if (ecamMachine != null) {
            arrayList = Utils.asList(ecamMachine.getWifiRecipe());
        } else {
            EcamMachine connectedEcam = getConnectedEcam();
            if (connectedEcam == null) {
                connectedEcam = this.mOfflineEcam;
            }
            if (connectedEcam != null) {
                arrayList = Utils.asList(connectedEcam.getBeanSystemRecipes());
            }
        }
        ArrayList<RecipeData> arrayList2 = new ArrayList<>();
        if (arrayList != null) {
            Iterator it2 = arrayList.iterator();
            while (it2.hasNext()) {
                RecipeData recipeData = (RecipeData) it2.next();
                if (recipeData != null && recipeData.getId() == 200) {
                    arrayList2.add(recipeData);
                }
            }
        }
        return arrayList2;
    }

    @Override // it.delonghi.IECamService
    public ArrayList<RecipeData> getAllClassicRecipes() {
        EcamMachine ecamMachine = this.ecamWifi;
        if (ecamMachine == null) {
            ecamMachine = getConnectedEcam();
        }
        if (ecamMachine == null) {
            ecamMachine = this.mOfflineEcam;
        }
        return ecamMachine == null ? new ArrayList<>() : Utils.asList(ecamMachine.getSelectedProfile().getRecipesData());
    }

    public ArrayList<RecipeData> getCustomRecipesV2() {
        ArrayList<RecipeData> arrayList = new ArrayList<>();
        EcamMachine ecamMachine = this.ecamWifi;
        if (ecamMachine == null) {
            ecamMachine = getConnectedEcam();
        }
        if (ecamMachine == null) {
            ecamMachine = this.mOfflineEcam;
        }
        ArrayList<RecipeData> arrayList2 = new ArrayList<>();
        if (ecamMachine == null) {
            return arrayList2;
        }
        ArrayList asList = Utils.asList(ecamMachine.getCustomRecipes());
        if (asList != null) {
            Iterator it2 = asList.iterator();
            while (it2.hasNext()) {
                RecipeData recipeData = (RecipeData) it2.next();
                if (recipeData != null) {
                    if (this.mDefaultsTable == null) {
                        this.mDefaultsTable = DefaultsTable.getInstance(getApplicationContext());
                    }
                    if (this.mDefaultsTable.getDefaultValuesForRecipe(ecamMachine.getOriginalName(), recipeData.getId()) != null && isRecipeCreated(recipeData) && recipeData.getPriority() != -1) {
                        arrayList.add(recipeData);
                    }
                }
            }
        }
        return arrayList;
    }

    @Override // it.delonghi.IECamService
    public ArrayList<RecipeData> getCustomRecipes() {
        return getCustomRecipesByEcam(getConnectedEcam(), getApplicationContext());
    }

    @Override // it.delonghi.IECamService
    public ArrayList<RecipeData> getCustomRecipesByEcam(EcamMachine ecamMachine, Context context) {
        ArrayList<RecipeData> arrayList = new ArrayList<>();
        if (ecamMachine == null) {
            if (this.mOfflineEcam == null) {
                return arrayList;
            }
            ecamMachine = this.mOfflineEcam;
        }
        ArrayList asList = Utils.asList(ecamMachine.getCustomRecipes());
        if (asList != null) {
            Iterator it2 = asList.iterator();
            while (it2.hasNext()) {
                RecipeData recipeData = (RecipeData) it2.next();
                if (recipeData != null) {
                    if (this.mDefaultsTable == null) {
                        this.mDefaultsTable = DefaultsTable.getInstance(context);
                    }
                    if (this.mDefaultsTable.getDefaultValuesForRecipe(ecamMachine.getOriginalName(), recipeData.getId()) != null && isRecipeCreated(recipeData)) {
                        arrayList.add(recipeData);
                    }
                }
            }
        }
        return arrayList;
    }

    @Override // it.delonghi.IECamService
    public RecipeData getNextCustomRecipe() {
        EcamMachine connectedEcam;
        if (DeLonghiManager.getInstance().CONNECTION_TYPE.equals(DeLonghiManager.getInstance().CONNECTION_WIFI)) {
            connectedEcam = DeLonghi.getInstance().getConnectService().ecamMachine();
        } else {
            connectedEcam = getConnectedEcam();
        }
        if (connectedEcam == null) {
            connectedEcam = this.mOfflineEcam;
        }
        if (connectedEcam != null) {
            SparseArray<RecipeData> customRecipes = connectedEcam.getCustomRecipes();
            for (int i = 0; i < connectedEcam.getCustomRecipesCnt(); i++) {
                RecipeData valueAt = customRecipes.valueAt(i);
                if (valueAt != null && valueAt.isCustomBeverage() && !isRecipeCreated(valueAt)) {
                    Iterator<ParameterModel> it2 = valueAt.getIngredients().iterator();
                    while (it2.hasNext()) {
                        ParameterModel next = it2.next();
                        if (next.getId() < 23) {
                            next.setDefValue(0);
                        }
                    }
                    return valueAt;
                }
            }
            return null;
        }
        return null;
    }

    private boolean isRecipeCreated(RecipeData recipeData) {
        Iterator<ParameterModel> it2 = recipeData.getIngredients().iterator();
        while (it2.hasNext()) {
            ParameterModel next = it2.next();
            if (next.getId() == IngredientsId.VISIBLE.getValue()) {
                return next.getDefValue() == 1;
            }
        }
        return false;
    }

    /* loaded from: classes2.dex */
    private class RetrieveEcamInfosFromDb extends AsyncTask<Void, Integer, Void> {
        private WeakReference<EcamServiceV2> mReference;

        RetrieveEcamInfosFromDb(EcamServiceV2 ecamServiceV2) {
            this.mReference = new WeakReference<>(ecamServiceV2);
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
                    MachineDefaults defaultValuesForMachine = EcamServiceV2.this.mDefaultsTable.getDefaultValuesForMachine(ecamMachine.getOriginalName());
                    if (ecamMachine2 != null) {
                        ecamMachine2.setProductCode(defaultValuesForMachine.getProductCode());
                        ecamMachine2.setAppModelId(defaultValuesForMachine.getAppModelId());
                        ecamMachine2.setCustomRecipesCnt(defaultValuesForMachine.getnCustomRecipes());
                        ecamMachine2.setBeanSystemRecipesCnt(defaultValuesForMachine.getnBeanSystemRecipes());
                        ecamMachine2.setDefaultRecipesCnt(defaultValuesForMachine.getnStandardRecipes());
                        ecamMachine2.setProfileNumbers(defaultValuesForMachine.getnProfiles());
                        ecamMachine2.setConnectionStatus(ecamMachine.getConnectionStatus());
                        ecamMachine2.setConnectionType(ecamMachine.getConnectionType());
                        ecamMachine2.setProtocolVersion(defaultValuesForMachine.getProtocolVersion());
                        ecamMachine2.setProfileIconSet(defaultValuesForMachine.getProfileIconSet());
                        ecamMachine2.setProtocolMinorVersion(defaultValuesForMachine.getProtocolMinorVersion());
                        ecamMachine2.setGrindersCount(defaultValuesForMachine.getnGrinders());
                        DeLonghiManager.getInstance().addEcamMachine(ecamMachine2);
                    } else {
                        ecamMachine.setProductCode(defaultValuesForMachine.getProductCode());
                        ecamMachine.setAppModelId(defaultValuesForMachine.getAppModelId());
                        ecamMachine.setCustomRecipesCnt(defaultValuesForMachine.getnCustomRecipes());
                        ecamMachine.setBeanSystemRecipesCnt(defaultValuesForMachine.getnBeanSystemRecipes());
                        ecamMachine.setDefaultRecipesCnt(defaultValuesForMachine.getnStandardRecipes());
                        ecamMachine.setModelName(defaultValuesForMachine.getName());
                        ecamMachine.setType(defaultValuesForMachine.getType());
                        ecamMachine.setConnectionType(defaultValuesForMachine.getConnectionType());
                        ecamMachine.setProtocolVersion(defaultValuesForMachine.getProtocolVersion());
                        ecamMachine.setProfileNumbers(defaultValuesForMachine.getnProfiles());
                        ecamMachine.setProfileIconSet(defaultValuesForMachine.getProfileIconSet());
                        ecamMachine.setProtocolMinorVersion(defaultValuesForMachine.getProtocolMinorVersion());
                        ecamMachine.setGrindersCount(defaultValuesForMachine.getnGrinders());
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
    private class RetrieveOfflineEcamInfo extends AsyncTask<Void, Integer, Void> {
        private WeakReference<EcamServiceV2> mReference;

        public RetrieveOfflineEcamInfo(EcamServiceV2 ecamServiceV2) {
            this.mReference = new WeakReference<>(ecamServiceV2);
        }

        /* JADX INFO: Access modifiers changed from: protected */
        @Override // android.os.AsyncTask
        public Void doInBackground(Void... voidArr) {
            if (this.mReference.get() != null) {
                DatabaseAdapter databaseAdapter = this.mReference.get().mDbAdapter;
                EcamMachine ecamMachine = this.mReference.get().mOfflineEcam;
                ecamMachine.setProfileNumbers(EcamServiceV2.this.mDefaultsTable.getDefaultValuesForMachine(ecamMachine.getOriginalName()).getnProfiles());
                try {
                    databaseAdapter.open();
                    if (databaseAdapter.getEcamMachine(Constants.OFFLINE_ECAM_ADDRESS) != null) {
                        SparseArray<RecipeData> retrieveRecipesData = databaseAdapter.retrieveRecipesData(ecamMachine.getAddress(), -1, 30);
                        if (retrieveRecipesData != null && retrieveRecipesData.size() != 0) {
                            ecamMachine.setCustomRecipes(retrieveRecipesData);
                            ecamMachine.getSelectedProfile().setRecipesData(databaseAdapter.retrieveRecipesData(ecamMachine.getAddress(), ecamMachine.getSelectedProfileIndex(), 30));
                        }
                    } else {
                        databaseAdapter.initEcamInDb(ecamMachine);
                    }
                    databaseAdapter.close();
                    return null;
                } catch (SQLException e) {
                    e.printStackTrace();
                    return null;
                }
            }
            return null;
        }
    }

    /* loaded from: classes2.dex */
    public static class InitMachineInfos extends AsyncTask<String, Integer, String> {
        private WeakReference<DefaultsTable> mDefaultsTable;
        private WeakReference<EcamServiceV2> mReference;
        private WeakReference<DeLonghiWifiConnectService> mReferenceWifi;

        public InitMachineInfos(EcamServiceV2 ecamServiceV2, DefaultsTable defaultsTable, DeLonghiWifiConnectService deLonghiWifiConnectService) {
            this.mReference = new WeakReference<>(ecamServiceV2);
            this.mDefaultsTable = new WeakReference<>(defaultsTable);
            this.mReferenceWifi = new WeakReference<>(deLonghiWifiConnectService);
        }

        /* JADX INFO: Access modifiers changed from: protected */
        /* JADX WARN: Multi-variable type inference failed */
        /* JADX WARN: Type inference failed for: r13v1 */
        /* JADX WARN: Type inference failed for: r13v2, types: [int] */
        @Override // android.os.AsyncTask
        public String doInBackground(String... strArr) {
            EcamMachine connectedEcam;
            char c = 0;
            boolean z = false;
            if (this.mReference.get() != null) {
                if (this.mReference.get().ecamWifi != null) {
                    connectedEcam = this.mReference.get().ecamWifi;
                } else {
                    connectedEcam = this.mReference.get().getConnectedEcam();
                }
                if (connectedEcam == null) {
                    return null;
                }
                connectedEcam.setTemperature(null);
                DatabaseAdapter databaseAdapter = this.mReference.get().ecamWifi != null ? DatabaseAdapter.getInstance(this.mReferenceWifi.get().getContext()) : this.mReference.get().mDbAdapter;
                MachineDefaults defaultValuesForMachine = this.mDefaultsTable.get().getDefaultValuesForMachine(connectedEcam.getOriginalName());
                DLog.d(EcamServiceV2.TAG, "InitMachineInfos");
                try {
                    databaseAdapter.open();
                    EcamMachine ecamMachine = databaseAdapter.getEcamMachine(strArr[0]);
                    connectedEcam.setProductCode(defaultValuesForMachine.getProductCode());
                    connectedEcam.setProtocolVersion(defaultValuesForMachine.getProtocolVersion());
                    connectedEcam.setProtocolMinorVersion(defaultValuesForMachine.getProtocolMinorVersion());
                    connectedEcam.setGrindersCount(defaultValuesForMachine.getnGrinders());
                    connectedEcam.setProfileIconSet(defaultValuesForMachine.getProfileIconSet());
                    connectedEcam.setCustomRecipesCnt(defaultValuesForMachine.getnCustomRecipes());
                    connectedEcam.setBeanSystemRecipesCnt(defaultValuesForMachine.getnBeanSystemRecipes());
                    connectedEcam.setDefaultRecipesCnt(defaultValuesForMachine.getnStandardRecipes());
                    connectedEcam.setProfileNumbers(defaultValuesForMachine.getnProfiles());
                    if (ecamMachine == null) {
                        SparseArray<RecipeData> sparseArray = new SparseArray<>();
                        SparseArray<RecipeData> sparseArray2 = new SparseArray<>();
                        int i = 0;
                        while (i < connectedEcam.getProfiles().size()) {
                            Profile profile = connectedEcam.getProfiles().get(connectedEcam.getProfiles().keyAt(i));
                            SparseArray<RecipeData> sparseArray3 = new SparseArray<>();
                            SparseArray<RecipeDefaults> recipeDefaults = defaultValuesForMachine.getRecipeDefaults();
                            DLog.e(EcamServiceV2.TAG, "INIT RECIPE FLOW : Profile " + profile + " recipe defaults size : " + recipeDefaults.size());
                            for (int i2 = z; i2 < recipeDefaults.size(); i2++) {
                                RecipeDefaults recipeDefaults2 = defaultValuesForMachine.getRecipeDefaults().get(defaultValuesForMachine.getRecipeDefaults().keyAt(i2));
                                if (recipeDefaults2.getId() < 200) {
                                    RecipeData recipeData = new RecipeData(recipeDefaults2.getId());
                                    recipeData.setCustomBeverage(z);
                                    recipeData.setIngredients(recipeDefaults2.getIngredients());
                                    sparseArray3.put(recipeDefaults2.getId(), recipeData);
                                    DLog.e(EcamServiceV2.TAG, "INIT RECIPE FLOW : classic ID " + recipeDefaults2.getId());
                                } else {
                                    RecipeData recipeData2 = new RecipeData(recipeDefaults2.getId());
                                    recipeData2.setIngredients(recipeDefaults2.getIngredients());
                                    if (recipeDefaults2.getId() >= 230) {
                                        recipeData2.setCustomBeverage(true);
                                        sparseArray.put(recipeDefaults2.getId(), recipeData2);
                                    } else {
                                        recipeData2.setCustomBeverage(false);
                                        sparseArray2.put(recipeDefaults2.getId(), recipeData2);
                                    }
                                    DLog.e(EcamServiceV2.TAG, "INIT RECIPE FLOW : custom ID " + recipeDefaults2.getId());
                                }
                                z = false;
                            }
                            profile.setRecipesData(sparseArray3);
                            i++;
                            z = false;
                        }
                        connectedEcam.setCustomRecipes(sparseArray);
                        connectedEcam.setBeanSystemRecipes(sparseArray2);
                        databaseAdapter.initEcamInDb(connectedEcam);
                        if (DeLonghi.getParameters() != null) {
                            databaseAdapter.insertIngredientsEntry(DeLonghi.getParameters());
                        }
                    } else {
                        SparseArray<Profile> retrieveProfiles = databaseAdapter.retrieveProfiles(ecamMachine.getAddress(), ecamMachine.getProtocolVersion());
                        connectedEcam.setProfiles(retrieveProfiles);
                        SparseArray<RecipeData> retrieveCustomRecipesData = databaseAdapter.retrieveCustomRecipesData(ecamMachine.getAddress(), defaultValuesForMachine.getnCustomRecipes() + defaultValuesForMachine.getnBeanSystemRecipes());
                        SparseArray<RecipeData> sparseArray4 = new SparseArray<>();
                        SparseArray<RecipeData> sparseArray5 = new SparseArray<>();
                        if (retrieveCustomRecipesData != null && retrieveCustomRecipesData.size() > 0) {
                            for (int i3 = 0; i3 < retrieveCustomRecipesData.size(); i3++) {
                                RecipeData recipeData3 = retrieveCustomRecipesData.get(retrieveCustomRecipesData.keyAt(i3));
                                if (recipeData3.getId() >= 230) {
                                    sparseArray4.put(recipeData3.getId(), recipeData3);
                                } else {
                                    sparseArray5.put(recipeData3.getId(), recipeData3);
                                }
                                DLog.e(EcamServiceV2.TAG, "STORED RECIPE FLOW : custom ID " + recipeData3.getId());
                            }
                        }
                        connectedEcam.setCustomRecipes(sparseArray4);
                        connectedEcam.setBeanSystemRecipes(sparseArray5);
                        if (retrieveProfiles != null) {
                            for (int i4 = 1; i4 <= retrieveProfiles.size(); i4++) {
                                retrieveProfiles.get(i4).setRecipesData(databaseAdapter.retrieveRecipesData(connectedEcam.getAddress(), i4, defaultValuesForMachine.getnStandardRecipes()));
                            }
                        }
                    }
                    databaseAdapter.close();
                } catch (SQLException e) {
                    e.printStackTrace();
                }
                c = 0;
            }
            return strArr[c];
        }

        /* JADX INFO: Access modifiers changed from: protected */
        @Override // android.os.AsyncTask
        public void onPostExecute(String str) {
            if (this.mReferenceWifi.get() == null) {
                this.mReference.get().onEcamInsertInDbFinished(str);
                return;
            }
            Bundle bundle = new Bundle();
            bundle.putString(Constants.ECAM_MACHINE_ADDRESS_EXTRA, str);
            EventBus.getDefault().post(new MachineConnectedEvent(bundle));
        }
    }

    /* loaded from: classes2.dex */
    public class StoreLastMonitorData extends AsyncTask<MonitorData, Void, Void> {
        private WeakReference<EcamServiceV2> mEcamService;

        StoreLastMonitorData(EcamServiceV2 ecamServiceV2) {
            this.mEcamService = new WeakReference<>(ecamServiceV2);
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

    public void setEcamWifi(EcamMachine ecamMachine) {
        this.ecamWifi = ecamMachine;
    }
}