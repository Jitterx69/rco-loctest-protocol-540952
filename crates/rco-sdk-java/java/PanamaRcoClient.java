
import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.util.concurrent.CompletableFuture;

/**
 * Hyper-Scale Java SDK for RCO Protocol.
 * Utilizes Java 22+ Foreign Function & Memory (FFM) API (Project Panama)
 * and Virtual Threads (Project Loom) for maximum performance.
 */
public class PanamaRcoClient {
    private static final SymbolLookup LOOKUP;
    private static final MethodHandle PROJECT_P14;
    private static final java.util.concurrent.Executor VIRTUAL_EXECUTOR = java.util.concurrent.Executors
            .newThreadPerTaskExecutor(Thread.ofVirtual().factory());

    static {
        // Load the native library using Panama's SymbolLookup
        System.loadLibrary("rco_sdk_java");
        LOOKUP = SymbolLookup.loaderLookup();

        // Bind the rco_panama_project_p14 function
        // int rco_panama_project_p14(const double* input, size_t len, long* output_ptr)
        PROJECT_P14 = Linker.nativeLinker().downcallHandle(
                LOOKUP.find("rco_panama_project_p14").orElseThrow(),
                FunctionDescriptor.of(ValueLayout.JAVA_INT,
                        ValueLayout.ADDRESS, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));
    }

    /**
     * Synchronous projection using Panama (Zero-overhead).
     */
    public long[] projectP14(double[] rewards) {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment inputSegment = arena.allocateFrom(ValueLayout.JAVA_DOUBLE, rewards);
            MemorySegment outputSegment = arena.allocate(ValueLayout.JAVA_LONG, rewards.length * 2);

            int result = (int) PROJECT_P14.invokeExact(inputSegment, (long) rewards.length, outputSegment);
            if (result != 0)
                throw new RuntimeException("RCO Projection Error: " + result);

            return outputSegment.toArray(ValueLayout.JAVA_LONG);
        } catch (Throwable t) {
            throw new RuntimeException("Panama invocation failed", t);
        }
    }

    /**
     * Asynchronous projection using Virtual Threads (Project Loom).
     * Hit the point of integral liberty: Concurrent ingestion at physical speeds.
     */
    public CompletableFuture<long[]> projectP14Async(double[] rewards) {
        return CompletableFuture.supplyAsync(() -> projectP14(rewards), VIRTUAL_EXECUTOR);
    }

    public static void main(String[] args) {
        PanamaRcoClient client = new PanamaRcoClient();
        double[] testBatch = new double[1000];
        for (int i = 0; i < 1000; i++)
            testBatch[i] = Math.random();

        // Warm up and test
        long[] result = client.projectP14(testBatch);
        System.out.println("Panama Project P14 Success. Batch Size: " + (result.length / 2));

        // Async test
        client.projectP14Async(testBatch).thenAccept(r -> {
            System.out.println("Async Virtual Thread Projection Complete.");
        }).join();
    }
}
