// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/security/ReentrancyGuardUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/security/PausableUpgradeable.sol";
import "../Interfaces/IBountyManager.sol";
import "../Interfaces/IReputationSystem.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

/**
 * @title BountyManagerV2
 * @dev Upgradeable version of BountyManager with enhanced features
 * @notice V2 improvements: Better consensus mechanism, dispute resolution, multi-sig support
 */
contract BountyManagerV2 is
    Initializable,
    UUPSUpgradeable,
    AccessControlUpgradeable,
    ReentrancyGuardUpgradeable,
    PausableUpgradeable
{
    // ============ ROLES ============

    bytes32 public constant UPGRADER_ROLE = keccak256("UPGRADER_ROLE");
    bytes32 public constant PAUSER_ROLE = keccak256("PAUSER_ROLE");
    bytes32 public constant OPERATOR_ROLE = keccak256("OPERATOR_ROLE");

    // ============ STATE VARIABLES ============

    IERC20 public threatToken;
    IReputationSystem public reputationSystem;

    uint256 public bountyCounter;
    uint256 public constant MIN_STAKE_AMOUNT = 10 * 10**18;
    uint256 public constant ANALYSIS_TIMEOUT = 24 hours;
    uint256 public constant CONSENSUS_THRESHOLD = 66;
    uint256 public constant PLATFORM_FEE_PERCENT = 5;
    uint256 public constant MIN_ANALYSES_TO_RESOLVE = 5;

    // V2 Additions
    uint256 public constant DISPUTE_PERIOD = 48 hours;
    uint256 public constant MIN_DISPUTE_STAKE = 50 * 10**18;
    uint256 public disputeCounter;

    address public feeCollector;

    // ============ ENUMS ============

    enum BountyStatus {
        Active,
        Resolved,
        Disputed,
        Cancelled
    }

    enum ThreatVerdict {
        Pending,
        Malicious,
        Benign
    }

    enum ArtifactType {
        File,
        URL,
        Hash,
        Domain,
        IP
    }

    enum DisputeStatus {
        Active,
        Resolved,
        Rejected
    }

    // ============ STRUCTS ============

    struct Bounty {
        uint256 id;
        address creator;
        string artifactHash;
        ArtifactType artifactType;
        uint256 rewardAmount;
        uint256 deadline;
        string description;
        BountyStatus status;
        ThreatVerdict consensusVerdict;
        uint256 totalStaked;
        uint256 analysisCount;
        uint256 createdAt;
    }

    struct Analysis {
        address analyst;
        ThreatVerdict verdict;
        uint256 confidence;
        uint256 stakeAmount;
        string analysisHash;
        uint256 submittedAt;
        bool rewarded;
    }

    // V2: Dispute mechanism
    struct Dispute {
        uint256 id;
        uint256 bountyId;
        address disputer;
        string reason;
        uint256 stakeAmount;
        DisputeStatus status;
        uint256 createdAt;
        uint256 resolvedAt;
        address resolver;
    }

    // ============ MAPPINGS ============

    mapping(uint256 => Bounty) public bounties;
    mapping(uint256 => mapping(address => Analysis)) public analyses;
    mapping(uint256 => address[]) public bountyAnalysts;
    mapping(address => uint256[]) public userBounties;
    mapping(uint256 => mapping(address => uint256)) public analystSubmissionIds;

    // V2 Additions
    mapping(uint256 => Dispute[]) public bountyDisputes;
    mapping(uint256 => bool) public isDisputed;
    mapping(address => bool) public trustedResolvers;

    // ============ EVENTS ============

    event BountyCreated(
        uint256 indexed bountyId,
        address indexed creator,
        string artifactHash,
        uint256 reward,
        uint256 deadline
    );

    event AnalysisSubmitted(
        uint256 indexed bountyId,
        address indexed analyst,
        ThreatVerdict verdict,
        uint256 stakeAmount,
        string analysisHash
    );

    event BountyResolved(
        uint256 indexed bountyId,
        ThreatVerdict consensusVerdict,
        uint256 totalReward,
        uint256 winnerCount
    );

    event StakeSlashed(
        uint256 indexed bountyId,
        address indexed analyst,
        uint256 slashedAmount
    );

    event RewardDistributed(
        uint256 indexed bountyId,
        address indexed analyst,
        uint256 rewardAmount
    );

    // V2 Events
    event DisputeCreated(
        uint256 indexed disputeId,
        uint256 indexed bountyId,
        address indexed disputer,
        string reason
    );

    event DisputeResolved(
        uint256 indexed disputeId,
        uint256 indexed bountyId,
        DisputeStatus status,
        address resolver
    );

    event TrustedResolverAdded(address indexed resolver);
    event TrustedResolverRemoved(address indexed resolver);

    // ============ MODIFIERS ============

    modifier validBounty(uint256 bountyId) {
        require(bountyId > 0 && bountyId <= bountyCounter, "Invalid bounty ID");
        _;
    }

    modifier bountyActive(uint256 bountyId) {
        require(bounties[bountyId].status == BountyStatus.Active, "Bounty not active");
        require(block.timestamp <= bounties[bountyId].deadline, "Bounty expired");
        _;
    }

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    // ============ INITIALIZER ============

    /**
     * @dev Initialize the contract (replaces constructor for upgradeable)
     */
    function initialize(
        address _threatToken,
        address _reputationSystem,
        address _feeCollector
    ) public initializer {
        require(_threatToken != address(0), "Invalid token address");
        require(_reputationSystem != address(0), "Invalid reputation system");
        require(_feeCollector != address(0), "Invalid fee collector");

        __UUPSUpgradeable_init();
        __AccessControl_init();
        __ReentrancyGuard_init();
        __Pausable_init();

        threatToken = IERC20(_threatToken);
        reputationSystem = IReputationSystem(_reputationSystem);
        feeCollector = _feeCollector;

        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(UPGRADER_ROLE, msg.sender);
        _grantRole(PAUSER_ROLE, msg.sender);
        _grantRole(OPERATOR_ROLE, msg.sender);
    }

    // ============ BOUNTY FUNCTIONS ============

    /**
     * @dev Create a new bounty (same as V1)
     */
    function createBounty(
        string memory artifactHash,
        ArtifactType artifactType,
        uint256 rewardAmount,
        uint256 deadline,
        string memory description
    ) external whenNotPaused returns (uint256) {
        require(bytes(artifactHash).length > 0, "Invalid artifact hash");
        require(rewardAmount > 0, "Reward must be positive");
        require(deadline > block.timestamp + 1 hours, "Deadline too soon");

        require(
            threatToken.transferFrom(msg.sender, address(this), rewardAmount),
            "Token transfer failed"
        );

        bountyCounter++;

        bounties[bountyCounter] = Bounty({
            id: bountyCounter,
            creator: msg.sender,
            artifactHash: artifactHash,
            artifactType: artifactType,
            rewardAmount: rewardAmount,
            deadline: deadline,
            description: description,
            status: BountyStatus.Active,
            consensusVerdict: ThreatVerdict.Pending,
            totalStaked: 0,
            analysisCount: 0,
            createdAt: block.timestamp
        });

        userBounties[msg.sender].push(bountyCounter);

        emit BountyCreated(bountyCounter, msg.sender, artifactHash, rewardAmount, deadline);

        return bountyCounter;
    }

    /**
     * @dev Submit analysis (same as V1)
     */
    function submitAnalysis(
        uint256 bountyId,
        ThreatVerdict verdict,
        uint256 confidence,
        uint256 stakeAmount,
        string memory analysisHash
    ) external validBounty(bountyId) bountyActive(bountyId) whenNotPaused {
        require(verdict != ThreatVerdict.Pending, "Invalid verdict");
        require(confidence > 0 && confidence <= 100, "Invalid confidence");
        require(stakeAmount >= MIN_STAKE_AMOUNT, "Insufficient stake");
        require(analyses[bountyId][msg.sender].analyst == address(0), "Already submitted");

        require(
            reputationSystem.getReputation(msg.sender) >= reputationSystem.getMinimumReputation(),
            "Insufficient reputation"
        );

        require(
            threatToken.transferFrom(msg.sender, address(this), stakeAmount),
            "Stake transfer failed"
        );

        analyses[bountyId][msg.sender] = Analysis({
            analyst: msg.sender,
            verdict: verdict,
            confidence: confidence,
            stakeAmount: stakeAmount,
            analysisHash: analysisHash,
            submittedAt: block.timestamp,
            rewarded: false
        });

        bountyAnalysts[bountyId].push(msg.sender);
        bounties[bountyId].totalStaked += stakeAmount;
        bounties[bountyId].analysisCount++;

        uint256 submissionId = reputationSystem.recordSubmission(
            msg.sender,
            bountyId,
            (verdict == ThreatVerdict.Malicious),
            stakeAmount,
            confidence
        );
        analystSubmissionIds[bountyId][msg.sender] = submissionId;

        emit AnalysisSubmitted(bountyId, msg.sender, verdict, stakeAmount, analysisHash);

        _checkAndResolveBounty(bountyId);
    }

    // ============ V2: DISPUTE FUNCTIONS ============

    /**
     * @dev Create a dispute for a resolved bounty
     * @param bountyId ID of the bounty to dispute
     * @param reason Reason for the dispute
     */
    function createDispute(
        uint256 bountyId,
        string memory reason
    ) external validBounty(bountyId) nonReentrant whenNotPaused {
        Bounty storage bounty = bounties[bountyId];
        require(bounty.status == BountyStatus.Resolved, "Bounty not resolved");
        require(
            block.timestamp <= bounty.deadline + DISPUTE_PERIOD,
            "Dispute period ended"
        );
        require(!isDisputed[bountyId], "Already disputed");
        require(bytes(reason).length > 0, "Reason required");

        // Require dispute stake
        require(
            threatToken.transferFrom(msg.sender, address(this), MIN_DISPUTE_STAKE),
            "Dispute stake transfer failed"
        );

        disputeCounter++;

        Dispute memory newDispute = Dispute({
            id: disputeCounter,
            bountyId: bountyId,
            disputer: msg.sender,
            reason: reason,
            stakeAmount: MIN_DISPUTE_STAKE,
            status: DisputeStatus.Active,
            createdAt: block.timestamp,
            resolvedAt: 0,
            resolver: address(0)
        });

        bountyDisputes[bountyId].push(newDispute);
        isDisputed[bountyId] = true;
        bounty.status = BountyStatus.Disputed;

        emit DisputeCreated(disputeCounter, bountyId, msg.sender, reason);
    }

    /**
     * @dev Resolve a dispute (trusted resolvers only)
     * @param bountyId ID of the disputed bounty
     * @param disputeIndex Index of the dispute in the array
     * @param accept Whether to accept the dispute
     */
    function resolveDispute(
        uint256 bountyId,
        uint256 disputeIndex,
        bool accept
    ) external validBounty(bountyId) nonReentrant {
        require(
            trustedResolvers[msg.sender] || hasRole(OPERATOR_ROLE, msg.sender),
            "Not authorized"
        );
        require(isDisputed[bountyId], "Bounty not disputed");
        require(disputeIndex < bountyDisputes[bountyId].length, "Invalid dispute index");

        Dispute storage dispute = bountyDisputes[bountyId][disputeIndex];
        require(dispute.status == DisputeStatus.Active, "Dispute not active");

        if (accept) {
            dispute.status = DisputeStatus.Resolved;
            // Return dispute stake plus bonus
            threatToken.transfer(dispute.disputer, dispute.stakeAmount + (dispute.stakeAmount / 2));

            // Re-open bounty for re-resolution or manual handling
            bounties[bountyId].status = BountyStatus.Active;
        } else {
            dispute.status = DisputeStatus.Rejected;
            // Slash dispute stake
            threatToken.transfer(feeCollector, dispute.stakeAmount);

            // Restore resolved status
            bounties[bountyId].status = BountyStatus.Resolved;
        }

        dispute.resolvedAt = block.timestamp;
        dispute.resolver = msg.sender;
        isDisputed[bountyId] = false;

        emit DisputeResolved(dispute.id, bountyId, dispute.status, msg.sender);
    }

    // ============ INTERNAL FUNCTIONS ============

    function _checkAndResolveBounty(uint256 bountyId) internal {
        Bounty storage bounty = bounties[bountyId];

        if (bounty.analysisCount >= MIN_ANALYSES_TO_RESOLVE * 2 || block.timestamp > bounty.deadline) {
            _resolveBountyInternal(bountyId);
        }
    }

    function _resolveBountyInternal(uint256 bountyId) internal {
        Bounty storage bounty = bounties[bountyId];
        address[] storage analysts = bountyAnalysts[bountyId];

        if (analysts.length == 0) {
            bounty.status = BountyStatus.Cancelled;
            threatToken.transfer(bounty.creator, bounty.rewardAmount);
            return;
        }

        (ThreatVerdict consensus, uint256 consensusCount) = _calculateConsensus(bountyId);

        bounty.consensusVerdict = consensus;
        bounty.status = BountyStatus.Resolved;

        for (uint256 i = 0; i < analysts.length; i++) {
            address analyst = analysts[i];
            uint256 submissionId = analystSubmissionIds[bountyId][analyst];
            if (submissionId > 0) {
                reputationSystem.resolveSubmission(submissionId, (consensus == ThreatVerdict.Malicious));
            }
        }

        _distributeRewards(bountyId, consensus, consensusCount);

        emit BountyResolved(bountyId, consensus, bounty.rewardAmount, consensusCount);
    }

    function _calculateConsensus(uint256 bountyId)
        internal
        view
        returns (ThreatVerdict consensus, uint256 consensusCount)
    {
        address[] storage analysts = bountyAnalysts[bountyId];
        uint256 maliciousCount = 0;
        uint256 benignCount = 0;
        uint256 totalWeight = 0;

        for (uint256 i = 0; i < analysts.length; i++) {
            Analysis storage analysis = analyses[bountyId][analysts[i]];
            uint256 weight = analysis.stakeAmount * analysis.confidence / 100;
            totalWeight += weight;

            if (analysis.verdict == ThreatVerdict.Malicious) {
                maliciousCount += weight;
            } else if (analysis.verdict == ThreatVerdict.Benign) {
                benignCount += weight;
            }
        }

        if (totalWeight == 0) return (ThreatVerdict.Pending, 0);

        uint256 maliciousPercent = (maliciousCount * 100) / totalWeight;

        if (maliciousPercent >= CONSENSUS_THRESHOLD) {
            consensus = ThreatVerdict.Malicious;
            consensusCount = _countCorrectAnalyses(bountyId, ThreatVerdict.Malicious);
        } else if ((benignCount * 100) / totalWeight >= CONSENSUS_THRESHOLD) {
            consensus = ThreatVerdict.Benign;
            consensusCount = _countCorrectAnalyses(bountyId, ThreatVerdict.Benign);
        } else {
            consensus = ThreatVerdict.Pending;
            consensusCount = 0;
        }
    }

    function _countCorrectAnalyses(uint256 bountyId, ThreatVerdict consensus)
        internal
        view
        returns (uint256 count)
    {
        address[] storage analysts = bountyAnalysts[bountyId];
        for (uint256 i = 0; i < analysts.length; i++) {
            if (analyses[bountyId][analysts[i]].verdict == consensus) {
                count++;
            }
        }
    }

    function _distributeRewards(
        uint256 bountyId,
        ThreatVerdict consensus,
        uint256 winnerCount
    ) internal {
        Bounty storage bounty = bounties[bountyId];
        address[] storage analysts = bountyAnalysts[bountyId];

        uint256 platformFee = (bounty.rewardAmount * PLATFORM_FEE_PERCENT) / 100;
        uint256 rewardPool = bounty.rewardAmount - platformFee;

        if (consensus == ThreatVerdict.Pending || winnerCount == 0) {
            uint256 refundAmount = bounty.rewardAmount - platformFee;
            threatToken.transfer(feeCollector, platformFee);
            threatToken.transfer(bounty.creator, refundAmount);

            for (uint256 i = 0; i < analysts.length; i++) {
                Analysis storage analysis = analyses[bountyId][analysts[i]];
                threatToken.transfer(analysts[i], analysis.stakeAmount);
            }
            return;
        }

        uint256 slashedAmount = _processSlashing(bountyId, consensus);
        rewardPool += slashedAmount;

        uint256 individualReward = rewardPool / winnerCount;

        for (uint256 i = 0; i < analysts.length; i++) {
            address analyst = analysts[i];
            Analysis storage analysis = analyses[bountyId][analyst];

            if (analysis.verdict == consensus && !analysis.rewarded) {
                analysis.rewarded = true;
                uint256 totalPayout = analysis.stakeAmount + individualReward;
                threatToken.transfer(analyst, totalPayout);

                emit RewardDistributed(bountyId, analyst, individualReward);
            }
        }

        threatToken.transfer(feeCollector, platformFee);
    }

    function _processSlashing(uint256 bountyId, ThreatVerdict consensus)
        internal
        returns (uint256 totalSlashed)
    {
        address[] storage analysts = bountyAnalysts[bountyId];

        for (uint256 i = 0; i < analysts.length; i++) {
            Analysis storage analysis = analyses[bountyId][analysts[i]];

            if (analysis.verdict != consensus) {
                totalSlashed += analysis.stakeAmount;
                emit StakeSlashed(bountyId, analysts[i], analysis.stakeAmount);
            }
        }
    }

    // ============ ADMIN FUNCTIONS ============

    function pause() external onlyRole(PAUSER_ROLE) {
        _pause();
    }

    function unpause() external onlyRole(PAUSER_ROLE) {
        _unpause();
    }

    function setFeeCollector(address _feeCollector) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(_feeCollector != address(0), "Invalid fee collector");
        feeCollector = _feeCollector;
    }

    function addTrustedResolver(address resolver) external onlyRole(OPERATOR_ROLE) {
        require(resolver != address(0), "Invalid address");
        trustedResolvers[resolver] = true;
        emit TrustedResolverAdded(resolver);
    }

    function removeTrustedResolver(address resolver) external onlyRole(OPERATOR_ROLE) {
        trustedResolvers[resolver] = false;
        emit TrustedResolverRemoved(resolver);
    }

    // ============ VIEW FUNCTIONS ============

    function getBounty(uint256 bountyId) external view validBounty(bountyId) returns (Bounty memory) {
        return bounties[bountyId];
    }

    function getAnalysis(uint256 bountyId, address analyst)
        external
        view
        validBounty(bountyId)
        returns (Analysis memory)
    {
        return analyses[bountyId][analyst];
    }

    function getBountyAnalysts(uint256 bountyId)
        external
        view
        validBounty(bountyId)
        returns (address[] memory)
    {
        return bountyAnalysts[bountyId];
    }

    function getBountyDisputes(uint256 bountyId)
        external
        view
        validBounty(bountyId)
        returns (Dispute[] memory)
    {
        return bountyDisputes[bountyId];
    }

    // ============ UPGRADE AUTHORIZATION ============

    function _authorizeUpgrade(address newImplementation)
        internal
        override
        onlyRole(UPGRADER_ROLE)
    {}

    /**
     * @dev Get version
     */
    function version() external pure returns (string memory) {
        return "2.0.0";
    }
}
