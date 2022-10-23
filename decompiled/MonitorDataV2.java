package it.delonghi.ecam.model;

import android.content.Context;
import android.os.Parcel;
import android.os.Parcelable;
import android.util.Base64;
import android.util.Log;
import com.google.common.base.Ascii;
import it.delonghi.R;
import it.delonghi.ayla.constant.AylaProperties;
import it.delonghi.ayla.dto.AylaDeviceDto;
import it.delonghi.ecam.model.enums.BeverageId;
import it.delonghi.ecam.model.enums.MachineAlarm;
import it.delonghi.ecam.model.enums.MachineKey;
import it.delonghi.ecam.model.enums.MachineLoad;
import it.delonghi.ecam.model.enums.MachineSwitch;
import it.delonghi.utils.DLog;
import it.delonghi.utils.Utils;
import java.util.ArrayList;

/* loaded from: classes2.dex */
public class MonitorDataV2 extends MonitorData {
    private static final int ACCESSORIES_DATA_2 = 4;
    private static final int AKEY0 = 4;
    private static final int AKEY1 = 5;
    private static final int AKEY2 = 6;
    private static final int AKEY3 = 7;
    private static final int AKEY4 = 8;
    private static final int AKEY5 = 5;
    private static final int AKEY6 = 6;
    private static final int BEVERAGE_TYPE_DATA_1 = 13;
    private static final int BEVERAGE_TYPE_DATA_2 = 23;
    private static final int COFFEE_INFUSER_POS_LSB_DATA_0 = 12;
    private static final int COFFEE_INFUSER_POS_LSB_DATA_2 = 16;
    private static final int COFFEE_INFUSER_POS_MSB_DATA_0 = 11;
    private static final int COFFEE_INFUSER_POS_MSB_DATA_2 = 15;
    private static final int COFFEE_POWDER_QTY_LSB = 18;
    private static final int COFFEE_POWDER_QTY_MSB = 17;
    private static final int COFFEE_WASTE_COUNTER_DATA_1 = 14;
    private static final int COFFEE_WASTE_COUNTER_DATA_2 = 24;
    private static final int CURRENT_WATER_FLOW_LSB_DATA_0 = 14;
    private static final int CURRENT_WATER_FLOW_LSB_DATA_2 = 18;
    private static final int CURRENT_WATER_FLOW_MSB_DATA_0 = 13;
    private static final int CURRENT_WATER_FLOW_MSB_DATA_2 = 17;
    private static final int DATA_0 = 0;
    private static final int DATA_1 = 1;
    public static final int DATA_2 = 2;
    private static final int DIAG0_DATA_1 = 4;
    private static final int DIAG0_DATA_2 = 7;
    private static final int DIAG1_DATA_1 = 5;
    private static final int DIAG1_DATA_2 = 8;
    private static final int DIAG2_DATA_1 = 5;
    private static final int DIAG2_DATA_2 = 12;
    private static final int DIAG3_DATA_1 = 5;
    private static final int DIAG3_DATA_2 = 13;
    private static final int FUNCTION_EXECUTION_PROGRESS_DATA_1 = 9;
    private static final int FUNCTION_EXECUTION_PROGRESS_DATA_2 = 10;
    private static final int FUNCTION_ONGOING_DATA_1 = 8;
    private static final int FUNCTION_ONGOING_DATA_2 = 9;
    private static final int HEATER_TEMP_DATA_1 = 11;
    private static final int HEATER_TEMP_DATA_2 = 21;
    private static final int LOADS0_DATA_1 = 6;
    private static final int LOADS0_DATA_2 = 13;
    private static final int LOADS1_DATA_1 = 7;
    private static final int LOADS1_DATA_2 = 14;
    private static final int MACHINE_MODEL_ID = 10;
    private static final int MAIN_BOARD_SW_RELEASE = 15;
    private static final int REQUESTED_WATER_QTY_LSB = 16;
    private static final int REQUESTED_WATER_QTY_MSB = 15;
    private static final int STEAMER_TEMP_DATA_1 = 12;
    private static final int STEAMER_TEMP_DATA_2 = 22;
    private int dataN;
    private long timestamp;
    private byte[] value;
    private static final String TAG = MonitorDataV2.class.getName();
    public static final Parcelable.Creator<MonitorDataV2> CREATOR = new Parcelable.Creator<MonitorDataV2>() { // from class: it.delonghi.ecam.model.MonitorDataV2.1
        /* JADX WARN: Can't rename method to resolve collision */
        @Override // android.os.Parcelable.Creator
        public MonitorDataV2 createFromParcel(Parcel parcel) {
            return new MonitorDataV2(parcel);
        }

        /* JADX WARN: Can't rename method to resolve collision */
        @Override // android.os.Parcelable.Creator
        public MonitorDataV2[] newArray(int i) {
            return new MonitorDataV2[i];
        }
    };

    @Override // it.delonghi.ecam.model.MonitorData, android.os.Parcelable
    public int describeContents() {
        return 0;
    }

    public MonitorDataV2(int i, byte[] bArr) {
        super(i, bArr);
        this.timestamp = System.currentTimeMillis();
        this.dataN = i;
        this.value = bArr;
    }

    public MonitorDataV2(int i, byte[] bArr, long j) {
        super(i, bArr, j);
        this.timestamp = j;
        this.dataN = i;
        this.value = bArr;
    }

    private MonitorDataV2(Parcel parcel) {
        super(parcel);
        this.dataN = parcel.readInt();
        parcel.readByteArray(this.value);
        this.timestamp = parcel.readLong();
    }

    public MonitorDataV2(int i, AylaDeviceDto aylaDeviceDto, Boolean bool) {
        super(i, aylaDeviceDto);
        try {
            this.dataN = i;
            if (aylaDeviceDto.getProperty(AylaProperties.MONITOR_MACHINE) != null && aylaDeviceDto.getProperty(AylaProperties.MONITOR_MACHINE).getValue() != null) {
                this.value = Base64.decode(aylaDeviceDto.getProperty(AylaProperties.MONITOR_MACHINE).getValue(), 2);
            }
            if (aylaDeviceDto.getProperty(AylaProperties.MONITOR_MACHINE_STRIKER) == null || aylaDeviceDto.getProperty(AylaProperties.MONITOR_MACHINE_STRIKER).getValue() == null) {
                return;
            }
            this.value = Base64.decode(aylaDeviceDto.getProperty(AylaProperties.MONITOR_MACHINE_STRIKER).getValue(), 2);
        } catch (Exception unused) {
            Log.e(TAG, "Monitor values are null");
        }
    }

    @Override // it.delonghi.ecam.model.MonitorData, android.os.Parcelable
    public void writeToParcel(Parcel parcel, int i) {
        parcel.writeInt(this.dataN);
        parcel.writeByteArray(this.value);
        parcel.writeLong(this.timestamp);
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public int getType() {
        return this.dataN;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public long getTimestamp() {
        return this.timestamp;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public void setTimestamp(long j) {
        this.timestamp = j;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public byte[] getValue() {
        return this.value;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public ArrayList<Integer> getPressedKeys() {
        if (this.dataN == 1) {
            return null;
        }
        ArrayList<Integer> arrayList = new ArrayList<>();
        byte[] bArr = this.value;
        long j = bArr[4] + (bArr[5] << 8) + (bArr[6] << 16) + (bArr[7] << Ascii.CAN) + (bArr[8] << 32);
        long j2 = 549755813888L;
        int i = 0;
        while (i < 40) {
            if ((j2 & j) != 0) {
                arrayList.add(Integer.valueOf(i));
            }
            i++;
            j2 >>= 1;
        }
        return arrayList;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public boolean isKeyPressed(MachineKey machineKey) {
        return (this.dataN == 1 || ((this.value[machineKey.getAkey() + 4] >> machineKey.getBitIndex()) & 1) == 0) ? false : true;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public ArrayList<Integer> getOnSwitches() {
        int i = 1;
        if (this.dataN == 1) {
            return null;
        }
        ArrayList<Integer> arrayList = new ArrayList<>();
        byte[] bArr = this.value;
        int i2 = bArr[5] + (bArr[6] << 8);
        int i3 = 0;
        while (i3 < 16) {
            if ((i & i2) != 0) {
                arrayList.add(Integer.valueOf(i3));
            }
            i3++;
            i <<= 1;
        }
        return arrayList;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public ArrayList<Integer> getOnSwitchesToShowUser() {
        int i = 1;
        if (this.dataN == 1) {
            return null;
        }
        ArrayList<Integer> arrayList = new ArrayList<>();
        int unsignedIntFromByte = Utils.getUnsignedIntFromByte(this.value[5]) + (Utils.getUnsignedIntFromByte(this.value[6]) << 8);
        int i2 = 0;
        while (i2 < 16) {
            if ((i & unsignedIntFromByte) != 0) {
                String str = TAG;
                Log.e(str, "Switch found: " + i2);
                if (!MachineSwitch.fromInt(i2).equals(MachineSwitch.IGNORE_SWITCH) && !MachineSwitch.fromInt(i2).equals(MachineSwitch.WATER_SPOUT) && !MachineSwitch.fromInt(i2).equals(MachineSwitch.IFD_CARAFFE) && !MachineSwitch.fromInt(i2).equals(MachineSwitch.CIOCCO_TANK)) {
                    arrayList.add(Integer.valueOf(i2));
                }
            }
            i2++;
            i <<= 1;
        }
        return arrayList;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public boolean isSwitchOn(MachineSwitch machineSwitch) {
        return (this.dataN == 1 || ((this.value[machineSwitch.getAkey() + 5] >> machineSwitch.getBitIndex()) & 1) == 0) ? false : true;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public int getCoffeeInfuserPos() {
        int i = this.dataN;
        if (i == 1) {
            return -1;
        }
        char c = i == 0 ? '\f' : (char) 16;
        char c2 = this.dataN == 2 ? (char) 11 : (char) 15;
        byte[] bArr = this.value;
        return bArr[c] + (bArr[c2] << 8);
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public int getWaterFlowQty() {
        int i = this.dataN;
        if (i == 1) {
            return -1;
        }
        char c = i == 0 ? (char) 14 : (char) 18;
        char c2 = this.dataN == 2 ? '\r' : (char) 17;
        byte[] bArr = this.value;
        return bArr[c2] + (bArr[c] << 8);
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public int getRequestedWaterQty() {
        if (this.dataN != 1) {
            return -1;
        }
        byte[] bArr = this.value;
        return bArr[16] + (bArr[15] << 8);
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public int getCoffeePowderQty() {
        if (this.dataN != 1) {
            return -1;
        }
        byte[] bArr = this.value;
        return bArr[18] + (bArr[17] << 8);
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public ArrayList<Integer> getActiveAlarms() {
        if (this.dataN == 0) {
            return null;
        }
        ArrayList<Integer> arrayList = new ArrayList<>();
        int i = 1;
        int unsignedIntFromByte = Utils.getUnsignedIntFromByte(this.value[this.dataN == 1 ? (char) 4 : (char) 7]) + (Utils.getUnsignedIntFromByte(this.value[this.dataN == 1 ? (char) 5 : '\b']) << 8) + (Utils.getUnsignedIntFromByte(this.value[12]) << 16) + (Utils.getUnsignedIntFromByte(this.value[13]) << 24);
        int i2 = 0;
        while (i2 < 32) {
            if ((i & unsignedIntFromByte) != 0) {
                String str = TAG;
                Log.e(str, "Alarm found: " + i2);
                if (!MachineAlarm.fromInt(i2).equals(MachineAlarm.IGNORE_ALARM)) {
                    arrayList.add(Integer.valueOf(i2));
                }
            }
            i2++;
            i <<= 1;
        }
        return arrayList;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public boolean isAlarmActive(MachineAlarm machineAlarm) {
        byte b;
        int i = this.dataN;
        if (i == 0) {
            return false;
        }
        if (i == 1) {
            b = this.value[machineAlarm.getDiag() + 4];
        } else {
            int diag = machineAlarm.getDiag();
            b = this.value[diag != 0 ? diag != 1 ? diag != 2 ? diag != 3 ? (char) 0 : '\r' : '\f' : '\b' : (char) 7];
        }
        String str = TAG;
        DLog.d(str, "isAlarmActive: " + Utils.byteToHex(b));
        boolean z = ((b >> machineAlarm.getBitIndex()) & 1) != 0;
        if (z) {
            DLog.e(TAG, "WATER TANK ALARM");
        }
        return z;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public ArrayList<Integer> getOnLoads() {
        if (this.dataN == 0) {
            return null;
        }
        ArrayList<Integer> arrayList = new ArrayList<>();
        MachineLoad.values();
        char c = this.dataN == 1 ? (char) 6 : '\r';
        char c2 = this.dataN == 1 ? (char) 7 : (char) 14;
        byte[] bArr = this.value;
        int i = bArr[c] + (bArr[c2] << 8);
        int i2 = 32768;
        int i3 = 0;
        while (i3 < 16) {
            if ((i2 & i) != 0) {
                arrayList.add(Integer.valueOf(i3));
            }
            i3++;
            i2 >>= 1;
        }
        return arrayList;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public boolean isLoadOn(MachineLoad machineLoad) {
        int i = this.dataN;
        if (i == 0) {
            return false;
        }
        return ((this.value[(i == 1 ? 6 : 13) + machineLoad.getLoad()] >> machineLoad.getBitIndex()) & 1) != 0;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public int getFunctionOngoing() {
        int i = this.dataN;
        if (i == 0) {
            return -1;
        }
        int i2 = i == 1 ? 8 : 9;
        byte[] bArr = this.value;
        if (bArr.length > i2) {
            return Utils.byteToInt(bArr[i2]);
        }
        return -1;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public int getFunctionExecutionProgress() {
        int i = this.dataN;
        if (i == 0) {
            return -1;
        }
        int i2 = i == 1 ? 9 : 10;
        byte[] bArr = this.value;
        if (bArr.length > i2) {
            return Utils.byteToInt(bArr[i2]);
        }
        return -1;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public int getMachineModelId() {
        if (this.dataN != 1) {
            return -1;
        }
        return this.value[10];
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public int getHeaterTemp() {
        int i = this.dataN;
        if (i == 0) {
            return -1;
        }
        byte b = this.value[i == 1 ? (char) 11 : (char) 21];
        if (b < 0) {
            return 0;
        }
        return b;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public int getSteamerTemp() {
        int i = this.dataN;
        if (i == 0) {
            return -1;
        }
        byte b = this.value[i == 1 ? '\f' : (char) 22];
        if (b < 0) {
            return 0;
        }
        return b;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public BeverageId getBeverageType() {
        int i = this.dataN;
        if (i == 0) {
            return null;
        }
        byte b = this.value[i == 1 ? '\r' : (char) 23];
        String str = TAG;
        DLog.d(str, "Beverage id: " + BeverageId.values()[b]);
        return BeverageId.values()[b];
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public int getCoffeeWasteCounter() {
        int i = this.dataN;
        if (i == 0) {
            return -1;
        }
        return this.value[i == 1 ? (char) 14 : (char) 24];
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public int getMainBoardSwRelease() {
        if (this.dataN == 0) {
            return -1;
        }
        return this.value[15];
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public int getDispensingPercentage() {
        if (this.dataN != 2) {
            return -1;
        }
        return Utils.byteToInt(this.value[11]);
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public String getDispensingStatus(Context context) {
        if (this.dataN == 0) {
            return null;
        }
        int functionOngoing = getFunctionOngoing();
        String[] stringArray = context.getResources().getStringArray(R.array.dispensing_status);
        if (functionOngoing > stringArray.length) {
            return null;
        }
        return stringArray[functionOngoing];
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public boolean isReadyToWork() {
        return this.dataN != 0 && getFunctionOngoing() == 7 && getFunctionExecutionProgress() == 0;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public boolean isInRecoveryMode() {
        return this.dataN != 0 && getFunctionOngoing() == 6;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public boolean isInStandBy() {
        return this.dataN != 0 && getFunctionOngoing() == 0;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public boolean isTurningOn() {
        return this.dataN != 0 && getFunctionOngoing() == 1;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public boolean isShuttingDown() {
        return this.dataN != 0 && getFunctionOngoing() == 2;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public boolean isShutDown() {
        return this.dataN != 0 && getFunctionOngoing() == 0;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public boolean isAccessoryPresent(int i) {
        return this.dataN != 1 && Utils.byteToInt(this.value[4]) == i;
    }

    @Override // it.delonghi.ecam.model.MonitorData
    public int getAccessoryPresent() {
        if (this.dataN == 1) {
            return -1;
        }
        return Utils.byteToInt(this.value[4]);
    }
}