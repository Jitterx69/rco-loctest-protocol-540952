--------------------------- MODULE RCO_SBA ---------------------------
EXTENDS Integers, Sequences, FiniteSets, TLC

CONSTANT Nodes, FaultyNodes, Roots

VARIABLES 
    nodeState,      \* [NodeID -> {"VOTING", "COMMITTED"}]
    voteRegistry,   \* [NodeID -> Root]
    finalityRoot    \* Global agreed-upon Root

TypeOK ==
    /\ nodeState \in [Nodes -> {"VOTING", "COMMITTED"}]
    /\ voteRegistry \in [Nodes -> Roots \cup {0}]
    /\ finalityRoot \in Roots \cup {0}

Init ==
    /\ nodeState = [n \in Nodes |-> "VOTING"]
    /\ voteRegistry = [n \in Nodes |-> 0]
    /\ finalityRoot = 0

(* Logic: Node casts a vote for a proposed root *)
CastVote(n, r) ==
    /\ nodeState[n] = "VOTING"
    /\ voteRegistry' = [voteRegistry EXCEPT ![n] = r]
    /\ UNCHANGED <<nodeState, finalityRoot>>

(* Logic: Consensus check (Supermajority > 2/3) *)
CommitConsensus(r) ==
    /\ finalityRoot = 0
    /\ Cardinality({n \in Nodes : voteRegistry[n] = r}) > (2 * Cardinality(Nodes)) \div 3
    /\ finalityRoot' = r
    /\ nodeState' = [n \in Nodes |-> "COMMITTED"]
    /\ UNCHANGED <<voteRegistry>>

Next ==
    \/ \exists n \in Nodes, r \in Roots : CastVote(n, r)
    \/ \exists r \in Roots : CommitConsensus(r)

(* Safety: All honest nodes must commit the same root *)
Safety ==
    \forall n1, n2 \in Nodes :
        (nodeState[n1] = "COMMITTED" /\ nodeState[n2] = "COMMITTED")
        => voteRegistry[n1] = voteRegistry[n2]

-----------------------------------------------------------------------------
