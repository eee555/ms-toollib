import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Paths;
import ms_toollib.*;

public class Demo {
    public static void main(String[] args) {
        System.out.println("=== ms_toollib Java Demo ===\n");

        int[][] board = {
            {1, 1, 1, 1, 1, 2, 2, 2},
            {1,-1, 1, 2,-1, 3,-1,-1},
            {1, 1, 1, 3,-1, 5, 3, 3},
            {0, 0, 0, 2,-1, 3,-1, 1},
            {0, 0, 0, 1, 1, 3, 2, 2},
            {0, 0, 0, 0, 0, 2,-1, 2},
            {0, 1, 1, 1, 0, 2,-1, 2},
            {0, 1,-1, 1, 0, 1, 1, 1},
        };

        System.out.println("-- Board Analysis --");
        System.out.println(" 3BV:  " + MsToollib.cal3BV(board));
        System.out.println(" ZiNi: " + MsToollib.calZini(board));
        System.out.println("HZiNi: " + MsToollib.calHzini(board));
        System.out.println("RZiNi: " + MsToollib.calRzini(board, 50));
        System.out.println(" Isl:  " + MsToollib.calIsl(board));
        System.out.println(" Op:   " + MsToollib.calOp(board));

        System.out.println("\n-- laymine (16x30, 99 mines) --");
        int[][] exp = MsToollib.laymine(16, 30, 99, 0, 0);
        System.out.println("  rows=" + exp.length + " cols=" + exp[0].length);
        int mines = 0;
        for (int[] row : exp)
            for (int c : row)
                if (c == -1) mines++;
        System.out.println("  actual mines=" + mines);

        System.out.println("\n-- Probability --");
        int[][] game = {
            {0, 0, 1,10,10,10,10,10},
            {0, 0, 2,10,10,10,10,10},
            {1, 1, 3,11,10,10,10,10},
            {10,10,4,10,10,10,10,10},
            {10,10,10,10,10,10,10,10},
            {10,10,10,10,10,10,10,10},
            {10,10,10,10,10,10,10,10},
            {10,10,10,10,10,10,10,10},
        };
        double[][] prob = MsToollib.calProbabilityOnboard(game, 10.0);
        for (double[] row : prob) {
            for (double v : row)
                System.out.printf("%.4f ", v);
            System.out.println();
        }

        // ─── Video demo ───
        System.out.println("\n-- EvfVideo --");
        String evfPath = "../../test_files/temp.evf";
        try {
            byte[] rawData = Files.readAllBytes(Paths.get(evfPath));
            try (EvfVideo video = new EvfVideo(rawData, "temp.evf")) {
                int rc = video.parse();
                System.out.println(" parse() -> " + rc);
                BaseVideo data = video.getData();
                System.out.println(" software=" + data.getSoftware());
                System.out.println(" player=" + data.getPlayer());
                System.out.println(" timeMs=" + data.getRtimeMs());
                System.out.println(" time=" + String.format("%.3f", data.getRtime()));
                System.out.println(" width=" + data.getWidth() + " height=" + data.getHeight() + " mines=" + data.getMineNum());
                System.out.println(" mode=" + data.getMode() + " level=" + data.getLevel());
                System.out.println(" nf=" + data.getNf() + " completed=" + data.getIsCompleted());

                data.analyse();
                System.out.println(" after analyse:");
                System.out.println(" bbbvSolved=" + data.getBbbvSolved());
                System.out.println(" ce=" + data.getCe());
                System.out.println(" corr=" + String.format("%.4f", data.getCorr()));
                System.out.println(" thrp=" + String.format("%.4f", data.getThrp()));
                System.out.println(" ioe=" + String.format("%.4f", data.getIoe()));
                System.out.println(" cl=" + data.getCl());
                System.out.println(" left=" + data.getLeft() + " right=" + data.getRight() + " dbl=" + data.getDouble());
                System.out.println(" flag=" + data.getFlag());
                System.out.println(" etime=" + String.format("%.3f", data.getEtime()));
                System.out.println(" path=" + String.format("%.1f", data.getPath()));
                System.out.println(" stnb=" + String.format("%.4f", data.getStnb()));
                System.out.println(" valid=" + data.isValid());
                System.out.println(" events=" + data.getEventCount());
                int n = Math.min(data.getEventCount(), 5);
                for (int i = 0; i < n; i++) {
                    System.out.println("  event[" + i + "]: t=" + String.format("%.3f", data.eventTime(i))
                        + " " + data.eventDesc(i));
                }
            }
        } catch (IOException e) {
            System.out.println(" (skip video demo: " + e.getMessage() + ")");
        }

        System.out.println("\n=== Done ===");
    }
}
