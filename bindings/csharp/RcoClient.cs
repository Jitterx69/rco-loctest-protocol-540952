using System;
using System.Runtime.InteropServices;
using System.Threading.Tasks;

namespace Rco.Sdk
{
    /// <summary>
    /// Hyper-Scale C# SDK for RCO Protocol.
    /// Optimized for .NET 8+ and Unity (Burst/IL2CPP).
    /// Hits the point of integral liberty via Zero-Allocation Spans.
    /// </summary>
    public unsafe class RcoClient
    {
        private const string LibName = "rco_sdk_csharp";

        // Native function declarations
        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int rco_project_p14_batch(double* input, UIntPtr count, long* outputHigh, long* outputLow);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int rco_construct_por_message(byte* weightHash, byte* merkleLink, byte* telemetry, UIntPtr telemetryLen, byte* output);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        private static extern int rco_verify_hardware_sovereignty();

        /// <summary>
        /// Projects a batch of rewards into the P14 lattice using zero-copy Spans.
        /// Zero heap allocation after initial array setup.
        /// </summary>
        public static void ProjectP14Batch(ReadOnlySpan<double> rewards, Span<long> outputHigh, Span<long> outputLow)
        {
            if (rewards.Length != outputHigh.Length || rewards.Length != outputLow.Length)
                throw new ArgumentException("Buffer lengths must match.");

            fixed (double* pRewards = rewards)
            fixed (long* pHigh = outputHigh)
            fixed (long* pLow = outputLow)
            {
                int result = rco_project_p14_batch(pRewards, (UIntPtr)rewards.Length, pHigh, pLow);
                if (result != 0) throw new Exception($"RCO P14 Projection Error: {result}");
            }
        }

        /// <summary>
        /// Asynchronous projection using ValueTask to minimize GC pressure.
        /// Ideal for high-frequency ingestion nodes.
        /// </summary>
        public static async ValueTask ProjectP14BatchAsync(double[] rewards, long[] outputHigh, long[] outputLow)
        {
            // Offload the native computation to the thread pool to keep the caller responsive
            await Task.Run(() => 
            {
                ProjectP14Batch(rewards.AsSpan(), outputHigh.AsSpan(), outputLow.AsSpan());
            });
        }

        /// <summary>
        /// Constructs a Proof of Reflexion (PoR) message hash with zero-copy.
        /// </summary>
        public static void ConstructPoRMessage(ReadOnlySpan<byte> weightHash, ReadOnlySpan<byte> merkleLink, ReadOnlySpan<byte> telemetry, Span<byte> output)
        {
            if (weightHash.Length != 32 || merkleLink.Length != 32)
                throw new ArgumentException("Hashes must be 32 bytes.");
            if (output.Length < 96)
                throw new ArgumentException("Output buffer must be at least 96 bytes.");

            fixed (byte* pWh = weightHash)
            fixed (byte* pMl = merkleLink)
            fixed (byte* pTel = telemetry)
            fixed (byte* pOut = output)
            {
                int result = rco_construct_por_message(pWh, pMl, pTel, (UIntPtr)telemetry.Length, pOut);
                if (result != 0) throw new Exception($"RCO PoR Construction Error: {result}");
            }
        }

        /// <summary>
        /// Hardware Sovereignty Check: Direct hardware-fused identity verification.
        /// </summary>
        public static bool IsHardwareSovereign()
        {
            return rco_verify_hardware_sovereignty() == 1;
        }
    }
}
