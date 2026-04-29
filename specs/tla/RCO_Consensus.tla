--------------------------- MODULE RCO_Consensus ---------------------------
EXTENDS Integers, Sequences, FiniteSets, TLC

CONSTANT Nodes, Threshold, MaxBatchHeight

VARIABLES 
    nodeState,  \* [NodeID -> State]
    walBuffer,  \* [NodeID -> Seq(Block)]
    quorumMap,  \* [BatchID -> Set(NodeID)]
    masterLedger \* Global view for safety check

States == {"IDLE", "SIGNING", "COMMITTING", "SYNCING"}

\* Invariants
TypeOK ==
    /\ nodeState \in [Nodes -> States]
    /\ \forall n \in Nodes : Len(walBuffer[n]) <= MaxBatchHeight

\* The No-Fork Invariant (The primary safety goal of Stage-I)
NoFork ==
    \forall i \in 1..MaxBatchHeight :
        Cardinality({b \in Range(masterLedger) : b.height = i}) <= 1

\* Helper operator to extract Range from a Sequence
Range(s) == {s[i] : i \in 1..Len(s)}

Init == 
    /\ nodeState = [n \in Nodes |-> "IDLE"]
    /\ walBuffer = [n \in Nodes |-> << >>]
    /\ quorumMap = [i \in 1..MaxBatchHeight |-> {}]
    /\ masterLedger = << >>

\* Action: Node receives a new telemetry batch from SDK
ReceiveBatch(n, batch) ==
    /\ nodeState[n] = "IDLE"
    /\ nodeState' = [nodeState EXCEPT ![n] = "SIGNING"]
    /\ walBuffer' = [walBuffer EXCEPT ![n] = Append(walBuffer[n], batch)]
    /\ UNCHANGED <<quorumMap, masterLedger>>

\* Action: Node creates a partial signature share
SignShare(n, idx) ==
    /\ nodeState[n] = "SIGNING"
    /\ quorumMap' = [quorumMap EXCEPT ![idx] = quorumMap[idx] \cup {n}]
    /\ nodeState' = [nodeState EXCEPT ![n] = "COMMITTING"]
    /\ UNCHANGED <<walBuffer, masterLedger>>

\* Action: Quorum reached, commit to persistent storage
CommitQuorum(idx) ==
    /\ Cardinality(quorumMap[idx]) >= Threshold
    /\ masterLedger' = Append(masterLedger, [height |-> idx])
    /\ nodeState' = [n \in Nodes |-> "IDLE"]
    /\ UNCHANGED <<walBuffer, quorumMap>>

Next ==
    \exists n \in Nodes :
        \/ \exists batch \in {[height |-> i] : i \in 1..MaxBatchHeight} : ReceiveBatch(n, batch)
        \/ \exists idx \in 1..MaxBatchHeight : SignShare(n, idx)
        \/ \exists idx \in 1..MaxBatchHeight : CommitQuorum(idx)

------------------------------------------------------------------------
