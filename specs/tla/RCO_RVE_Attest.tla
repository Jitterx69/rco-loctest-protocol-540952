--------------------------- MODULE RCO_RVE_Attest ---------------------------
EXTENDS Integers, Sequences, FiniteSets, TLC

CONSTANT Nodes

VARIABLE 
    enclaveState, 
    quoteRegistry, 
    attestationChain

TypeOK == 
    /\ enclaveState \in [Nodes -> {"SECURE", "COMPROMISED", "UNINITIALIZED"}]
    /\ quoteRegistry \in [Nodes -> 1..100] \* Mocked quotes

Init ==
    /\ enclaveState = [n \in Nodes |-> "UNINITIALIZED"]
    /\ quoteRegistry = [n \in Nodes |-> 0]
    /\ attestationChain = <<>>

(* Logic: Verify Local Enclave *)
LocalAttest(n) ==
    /\ enclaveState[n] = "UNINITIALIZED"
    /\ enclaveState' = [enclaveState EXCEPT ![n] = "SECURE"]
    /\ quoteRegistry' = [quoteRegistry EXCEPT ![n] = 42] \* Mocked valid quote
    /\ UNCHANGED <<attestationChain>>

(* Logic: Recursive Cross-Attestation *)
CrossAttest(n1, n2) ==
    /\ enclaveState[n1] = "SECURE"
    /\ enclaveState[n2] = "SECURE"
    /\ attestationChain' = Append(attestationChain, <<n1, n2, quoteRegistry[n1] + quoteRegistry[n2]>>)
    /\ UNCHANGED <<enclaveState, quoteRegistry>>

Next ==
    \/ \exists n \in Nodes : LocalAttest(n)
    \/ \exists n1, n2 \in Nodes : n1 /= n2 /\ CrossAttest(n1, n2)

-----------------------------------------------------------------------------
