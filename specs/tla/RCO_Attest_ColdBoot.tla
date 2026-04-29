--------------------------- MODULE RCO_Attest_ColdBoot ---------------------------
EXTENDS Integers, Sequences, FiniteSets, TLC

CONSTANT Nodes, Quotes, CRL_List

VARIABLE 
    nodeCerts, 
    verificationStatus

TypeOK ==
    /\ nodeCerts \in [Nodes -> Quotes]
    /\ verificationStatus \in [Nodes -> {"SECURE", "REVOKED", "PENDING"}]

Init ==
    /\ nodeCerts = [n \in Nodes |-> 1] \* Mock cert
    /\ verificationStatus = [n \in Nodes |-> "PENDING"]

(* Action: Update CRL and Re-Attest *)
UpdateAndVerify ==
    /\ \forall n \in Nodes :
        IF nodeCerts[n] \in CRL_List 
        THEN verificationStatus' = [verificationStatus EXCEPT ![n] = "REVOKED"]
        ELSE verificationStatus' = [verificationStatus EXCEPT ![n] = "SECURE"]
    /\ UNCHANGED <<nodeCerts>>

Next ==
    UpdateAndVerify

(* Assertion: No node is SECURE if its Cert is in CRL_List *)
LivenessProperty == 
    \forall n \in Nodes : 
        (nodeCerts[n] \in CRL_List) => (verificationStatus[n] /= "SECURE")

-----------------------------------------------------------------------------
