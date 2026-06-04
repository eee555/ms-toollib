package ms_toollib;

public class RmvVideo implements AutoCloseable {
    private long nativePtr;
    private BaseVideo dataView;

    public RmvVideo(String fileName) { nativePtr = nativeNew(fileName); }
    public RmvVideo(byte[] rawData, String fileName) { nativePtr = nativeNewFromData(rawData, fileName); }

    public int parse() { return nativeParse(nativePtr); }

    public BaseVideo getData() {
        if (dataView == null) dataView = new BaseVideo(nativeDataPtr(nativePtr));
        return dataView;
    }

    @Override public void close() {
        if (nativePtr != 0) { nativeFree(nativePtr); nativePtr = 0; dataView = null; }
    }

    static native long nativeNew(String fileName);
    static native long nativeNewFromData(byte[] data, String fileName);
    static native void nativeFree(long ptr);
    static native int nativeParse(long ptr);
    static native long nativeDataPtr(long ptr);
}
