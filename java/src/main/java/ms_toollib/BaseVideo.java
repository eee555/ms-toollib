package ms_toollib;

public class BaseVideo {
    long nativePtr;

    BaseVideo(long ptr) { this.nativePtr = ptr; }

    public void analyse() { nativeAnalyse(nativePtr); }

    public double getRtime() { return nativeGetRtime(nativePtr); }
    public int getRtimeMs() { return nativeGetRtimeMs(nativePtr); }
    public double getEtime() { return nativeGetEtime(nativePtr); }
    public int getLeft() { return nativeGetLeft(nativePtr); }
    public int getRight() { return nativeGetRight(nativePtr); }
    public int getDouble() { return nativeGetDouble(nativePtr); }
    public int getCl() { return nativeGetCl(nativePtr); }
    public int getFlag() { return nativeGetFlag(nativePtr); }
    public int getBbbvSolved() { return nativeGetBbbvSolved(nativePtr); }
    public int getCe() { return nativeGetCe(nativePtr); }
    public double getCorr() { return nativeGetCorr(nativePtr); }
    public double getThrp() { return nativeGetThrp(nativePtr); }
    public double getIoe() { return nativeGetIoe(nativePtr); }
    public double getPath() { return nativeGetPath(nativePtr); }
    public double getStnb() { return nativeGetStnb(nativePtr); }
    public int getMouseState() { return nativeGetMouseState(nativePtr); }
    public int getCurrentEventId() { return nativeGetCurrentEventId(nativePtr); }
    public byte setCurrentEventId(int id) { return nativeSetCurrentEventId(nativePtr, id); }
    public double getCurrentTime() { return nativeGetCurrentTime(nativePtr); }
    public void setCurrentTime(double t) { nativeSetCurrentTime(nativePtr, t); }
    public int getEventCount() { return nativeGetEventCount(nativePtr); }
    public byte isValid() { return nativeIsValid(nativePtr); }
    public double eventTime(int idx) { return nativeEventTime(nativePtr, idx); }
    public String eventDesc(int idx) { return nativeEventDesc(nativePtr, idx); }
    public int getWidth() { return nativeGetWidth(nativePtr); }
    public int getHeight() { return nativeGetHeight(nativePtr); }
    public int getMineNum() { return nativeGetMineNum(nativePtr); }
    public int getMode() { return nativeGetMode(nativePtr); }
    public int getLevel() { return nativeGetLevel(nativePtr); }
    public boolean getNf() { return nativeGetNf(nativePtr); }
    public boolean getIsCompleted() { return nativeGetIsCompleted(nativePtr); }
    public String getSoftware() { return nativeGetSoftware(nativePtr); }
    public String getPlayer() { return nativeGetPlayer(nativePtr); }
    public int[][] getGameBoard() { return nativeGetGameBoard(nativePtr); }

    static native void nativeAnalyse(long ptr);
    static native double nativeGetRtime(long ptr);
    static native int nativeGetRtimeMs(long ptr);
    static native double nativeGetEtime(long ptr);
    static native int nativeGetLeft(long ptr);
    static native int nativeGetRight(long ptr);
    static native int nativeGetDouble(long ptr);
    static native int nativeGetCl(long ptr);
    static native int nativeGetFlag(long ptr);
    static native int nativeGetBbbvSolved(long ptr);
    static native int nativeGetCe(long ptr);
    static native double nativeGetCorr(long ptr);
    static native double nativeGetThrp(long ptr);
    static native double nativeGetIoe(long ptr);
    static native double nativeGetPath(long ptr);
    static native double nativeGetStnb(long ptr);
    static native int nativeGetMouseState(long ptr);
    static native int nativeGetCurrentEventId(long ptr);
    static native byte nativeSetCurrentEventId(long ptr, int id);
    static native double nativeGetCurrentTime(long ptr);
    static native void nativeSetCurrentTime(long ptr, double t);
    static native int nativeGetEventCount(long ptr);
    static native byte nativeIsValid(long ptr);
    static native double nativeEventTime(long ptr, int idx);
    static native String nativeEventDesc(long ptr, int idx);
    static native int nativeGetWidth(long ptr);
    static native int nativeGetHeight(long ptr);
    static native int nativeGetMineNum(long ptr);
    static native int nativeGetMode(long ptr);
    static native int nativeGetLevel(long ptr);
    static native boolean nativeGetNf(long ptr);
    static native boolean nativeGetIsCompleted(long ptr);
    static native String nativeGetSoftware(long ptr);
    static native String nativeGetPlayer(long ptr);
    static native int[][] nativeGetGameBoard(long ptr);
}
