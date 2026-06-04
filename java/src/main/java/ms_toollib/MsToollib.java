package ms_toollib;

public final class MsToollib {
    static {
        System.loadLibrary("ms_toollib_jni");
    }

    private MsToollib() {}

    public static native int cal3BV(int[][] board);
    public static native int calZini(int[][] board);
    public static native int calHzini(int[][] board);
    public static native int calRzini(int[][] board, int nIter);
    public static native int calIsl(int[][] board);
    public static native int calOp(int[][] board);
    public static native int[][] laymine(int row, int col, int mineNum, int x0, int y0);
    public static native double[][] calProbabilityOnboard(int[][] gameBoard, double mineNum);
}
