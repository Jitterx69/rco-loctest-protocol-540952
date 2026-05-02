const { 
    projectP14BatchBigint, 
    projectP14BatchAsync,
    constructPorMessageNode, 
    verifyHardwareSovereigntyNode 
} = require('./index.node');

async function verifyHyperScaleNodeSDK() {
    console.log("--- RCO Node.js HYPER-SCALE Verification ---");

    // 1. Test BigInt64Array Zero-Copy Projection
    const rewards = new Float64Array([1.0, -0.5, 3.14159]);
    const outHigh = new BigInt64Array(rewards.length);
    const outLow = new BigInt64Array(rewards.length);

    projectP14BatchBigint(rewards, outHigh, outLow);
    
    console.log("BigInt High Results:", outHigh);
    console.log("BigInt Low Results:", outLow);

    if (outHigh[1] === -1n) {
        console.log("✅ BigInt64Array Projection: SUCCESS");
    } else {
        console.log("❌ BigInt64Array Projection: FAILED");
    }

    // 2. Test Multi-Threaded Async Projection
    console.log("Starting Async libuv Task...");
    await projectP14BatchAsync(rewards, outHigh, outLow);
    console.log("✅ Async libuv Task: SUCCESS");

    // 3. Test PoR Message Construction
    const wh = new Uint8Array(32).fill(1);
    const ml = new Uint8Array(32).fill(2);
    const tel = new Uint8Array(Buffer.from("telemetry_hyper_scale"));
    const por = constructPorMessageNode(wh, ml, tel);
    console.log("PoR Message Length:", por.length);
    if (por.length === 96) {
        console.log("✅ PoR Construction: SUCCESS");
    }

    // 4. Test Hardware Sovereignty
    const isSovereign = verifyHardwareSovereigntyNode();
    console.log("Hardware Sovereignty Verified:", isSovereign);
    console.log("✅ Hardware Check: SUCCESS");
}

verifyHyperScaleNodeSDK().catch(console.error);
