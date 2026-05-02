import java.util.Arrays;

public class RcoClient {
    static {
        System.loadLibrary("rco_sdk_java");
    }

    /**
     * Projects a batch of rewards into the P14 lattice.
     */
    public native long[] projectP14Batch(double[] rewards);

    /**
     * Constructs a Proof of Reflexion (PoR) message hash.
     */
    public native byte[] constructPoRMessage(byte[] weightHash, byte[] merkleLink, byte[] telemetry);

    /**
     * Verifies if the current host provides a valid Hardware Root of Trust.
     */
    public native boolean verifyHardwareSovereignty();

    public static void main(String[] args) {
        RcoClient client = new RcoClient();
        double[] testRewards = {1.0, -0.5, 3.14159};
        long[] result = client.projectP14Batch(testRewards);
        
        System.out.println("Projected Batch Size: " + (result.length / 2));
        for (int i = 0; i < result.length; i += 2) {
            System.out.printf("Reward %d: High=%d, Low=%d\n", i/2, result[i], result[i+1]);
        }
    }
}
