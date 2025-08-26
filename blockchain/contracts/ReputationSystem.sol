// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "./interfaces/IReputationSystem.sol";

/**
 * @title ReputationSystem
 * @dev Manages reputation scores for security analysts and engines in the Nexus-Security platform
 * @notice This contract tracks accuracy, participation, and calculates weighted reputation scores
 */
abstract contract ReputationSystem is IReputationSystem, AccessControl {

    bytes32 public constant BOUNTY_MANAGER_ROLE = keccak256("BOUNTY_MANAGER_ROLE");
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");

    // Constants for reputation calculations
    uint256 public constant INITIAL_REPUTATION = 100;
    uint256 public constant MAX_REPUTATION = 1000;
    uint256 public constant MIN_REPUTATION = 0;
    uint256 public constant ACCURACY_WEIGHT = 70; // 70% weight for accuracy
    uint256 public constant PARTICIPATION_WEIGHT = 20; // 20% weight for participation
    uint256 public constant CONSISTENCY_WEIGHT = 10; // 10% weight for consistency
    uint256 public constant DECAY_RATE = 5; // 5% decay per month for inactive users
    uint256 public constant MIN_SUBMISSIONS_FOR_RATING = 10;

    // Structs
    struct AnalystProfile {
        uint256 reputation;
        uint256 totalSubmissions;
        uint256 correctPredictions;
        uint256 falsePositives;
        uint256 falseNegatives;
        uint256 totalStakeAmount;
        uint256 totalRewards;
        uint256 lastActiveTimestamp;
        uint256 consecutiveCorrect;
        uint256 maxConsecutiveCorrect;
        bool isActive;
        AnalystCategory category;
    }

    struct SubmissionRecord {
        address analyst;
        uint256 bountyId;
        bool prediction; // true for malicious, false for benign
        bool actualResult;
        uint256 stakeAmount;
        uint256 timestamp;
        bool isResolved;
        uint256 confidenceScore; // 0-100
    }

    struct ReputationTier {
        uint256 minReputation;
        uint256 maxStakeMultiplier;
        uint256 rewardBonus; // percentage bonus
        string tierName;
    }

    enum AnalystCategory {
        Human,
        AutomatedEngine,
        HybridSystem
    }

    // State variables
    mapping(address => AnalystProfile) public analysts;
    mapping(uint256 => SubmissionRecord) public submissions;
    mapping(address => uint256[]) public analystSubmissions;
    mapping(uint256 => ReputationTier) public reputationTiers;
    
    address[] public activeAnalysts;
    uint256 public nextSubmissionId;
    uint256 public totalAnalysts;
    uint256 public decayTimestamp;

    // // Events
    // event AnalystRegistered(address indexed analyst, AnalystCategory category);
    // event SubmissionRecorded(uint256 indexed submissionId, address indexed analyst, uint256 bountyId);
    // event SubmissionResolved(uint256 indexed submissionId, bool correct, uint256 reputationChange);
    // event ReputationUpdated(address indexed analyst, uint256 oldReputation, uint256 newReputation);
    // event TierUpdated(address indexed analyst, uint256 newTier);
    // event ReputationDecayed(address indexed analyst, uint256 decayAmount);

    constructor() {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(ADMIN_ROLE, msg.sender);
        
        // Initialize reputation tiers
        _initializeReputationTiers();
        
        decayTimestamp = block.timestamp;
        nextSubmissionId = 1;
    }

    /**
     * @dev Register a new analyst in the system
     * @param analyst Address of the analyst
     * @param category Type of analyst (Human, AutomatedEngine, HybridSystem)
     */
    function registerAnalyst(address analyst, AnalystCategory category) 
        external 
        override 
        onlyRole(BOUNTY_MANAGER_ROLE) 
    {
        require(analyst != address(0), "Invalid analyst address");
        require(!analysts[analyst].isActive, "Analyst already registered");

        analysts[analyst] = AnalystProfile({
            reputation: INITIAL_REPUTATION,
            totalSubmissions: 0,
            correctPredictions: 0,
            falsePositives: 0,
            falseNegatives: 0,
            totalStakeAmount: 0,
            totalRewards: 0,
            lastActiveTimestamp: block.timestamp,
            consecutiveCorrect: 0,
            maxConsecutiveCorrect: 0,
            isActive: true,
            category: category
        });

        activeAnalysts.push(analyst);
        totalAnalysts += 1;

        emit AnalystRegistered(analyst, category);
    }

    /**
     * @dev Record a new submission from an analyst
     * @param analyst Address of the analyst
     * @param bountyId ID of the bounty
     * @param prediction Analyst's prediction (true for malicious, false for benign)
     * @param stakeAmount Amount staked on this prediction
     * @param confidenceScore Confidence score (0-100)
     */
    function recordSubmission(
        address analyst,
        uint256 bountyId,
        bool prediction,
        uint256 stakeAmount,
        uint256 confidenceScore
    ) external override onlyRole(BOUNTY_MANAGER_ROLE) returns (uint256) {
        require(analysts[analyst].isActive, "Analyst not registered");
        require(confidenceScore <= 100, "Invalid confidence score");

        uint256 submissionId = nextSubmissionId;
        nextSubmissionId += 1;

        submissions[submissionId] = SubmissionRecord({
            analyst: analyst,
            bountyId: bountyId,
            prediction: prediction,
            actualResult: false, // Will be set when resolved
            stakeAmount: stakeAmount,
            timestamp: block.timestamp,
            isResolved: false,
            confidenceScore: confidenceScore
        });

        analystSubmissions[analyst].push(submissionId);
        analysts[analyst].totalSubmissions += 1;
        analysts[analyst].totalStakeAmount += stakeAmount;
        analysts[analyst].lastActiveTimestamp = block.timestamp;

        emit SubmissionRecorded(submissionId, analyst, bountyId);
        return submissionId;
    }

    /**
     * @dev Resolve a submission and update reputation
     * @param submissionId ID of the submission
     * @param actualResult The actual result (true for malicious, false for benign)
     */
    function resolveSubmission(uint256 submissionId, bool actualResult) 
        external 
        override 
        onlyRole(BOUNTY_MANAGER_ROLE) 
    {
        SubmissionRecord storage submission = submissions[submissionId];
        require(!submission.isResolved, "Submission already resolved");
        require(submission.analyst != address(0), "Invalid submission");

        submission.actualResult = actualResult;
        submission.isResolved = true;

        bool isCorrect = submission.prediction == actualResult;
        _updateAnalystReputation(submission.analyst, submissionId, isCorrect);

        emit SubmissionResolved(submissionId, isCorrect, 0);
    }

    /**
     * @dev Update analyst reputation based on submission result
     */
    function _updateAnalystReputation(address analyst, uint256 submissionId, bool isCorrect) internal {
        AnalystProfile storage profile = analysts[analyst];
        SubmissionRecord storage submission = submissions[submissionId];
        
        uint256 oldReputation = profile.reputation;
        uint256 reputationChange = 0;

        if (isCorrect) {
            profile.correctPredictions += 1;
            profile.consecutiveCorrect += 1;
            
            if (profile.consecutiveCorrect > profile.maxConsecutiveCorrect) {
                profile.maxConsecutiveCorrect = profile.consecutiveCorrect;
            }

            // Calculate reputation gain based on confidence and stake
            reputationChange = _calculateReputationGain(analyst, submissionId);
            profile.reputation += reputationChange;
            
        } else {
            profile.consecutiveCorrect = 0;
            
            // Track false positives and false negatives
            if (submission.prediction && !submission.actualResult) {
                profile.falsePositives += 1;
            } else if (!submission.prediction && submission.actualResult) {
                profile.falseNegatives += 1;
            }

            // Calculate reputation loss
            reputationChange = _calculateReputationLoss(analyst, submissionId);
            if (profile.reputation > reputationChange) {
                profile.reputation -= reputationChange;
            } else {
                profile.reputation = MIN_REPUTATION;
            }
        }

        // Cap reputation at maximum
        if (profile.reputation > MAX_REPUTATION) {
            profile.reputation = MAX_REPUTATION;
        }

        emit ReputationUpdated(analyst, oldReputation, profile.reputation);
    }

    /**
     * @dev Calculate reputation gain for correct predictions
     */
    function _calculateReputationGain(address analyst, uint256 submissionId) internal view returns (uint256) {
        AnalystProfile memory profile = analysts[analyst];
        SubmissionRecord memory submission = submissions[submissionId];
        
        // Base gain
        uint256 baseGain = 5;
        
        // Confidence bonus (higher confidence = higher reward if correct)
        uint256 confidenceBonus = submission.confidenceScore / 20; // 0-5 bonus
        
        // Consistency bonus
        uint256 consistencyBonus = profile.consecutiveCorrect / 5; // Bonus for streaks
        
        // Stake-based bonus (higher stake = higher reward)
        uint256 stakeBonus = submission.stakeAmount / 1000; // Adjust divisor as needed
        if (stakeBonus > 10) stakeBonus = 10; // Cap at 10
        
        return baseGain + confidenceBonus + consistencyBonus + stakeBonus;
    }

    /**
     * @dev Calculate reputation loss for incorrect predictions
     */
    function _calculateReputationLoss(address analyst, uint256 submissionId) internal view returns (uint256) {
        SubmissionRecord memory submission = submissions[submissionId];
        
        // Base loss
        uint256 baseLoss = 10;
        
        // Confidence penalty (higher confidence = higher penalty if wrong)
        uint256 confidencePenalty = submission.confidenceScore / 10; // 0-10 penalty
        
        // Stake-based penalty
        uint256 stakePenalty = submission.stakeAmount / 500; // Adjust divisor as needed
        if (stakePenalty > 15) stakePenalty = 15; // Cap at 15
        
        return baseLoss + confidencePenalty + stakePenalty;
    }

    /**
     * @dev Get analyst's current reputation
     * @param analyst Address of the analyst
     * @return Current reputation score
     */
    function getReputation(address analyst) external view override returns (uint256) {
        return analysts[analyst].reputation;
    }

    /**
     * @dev Get analyst's accuracy rate
     * @param analyst Address of the analyst
     * @return Accuracy rate as percentage (0-100)
     */
    function getAccuracyRate(address analyst) external view override returns (uint256) {
        AnalystProfile memory profile = analysts[analyst];
        if (profile.totalSubmissions == 0) return 0;
        
        return profile.correctPredictions * 100 / profile.totalSubmissions;
    }

    /**
     * @dev Get comprehensive analyst statistics
     * @param analyst Address of the analyst
     * @return AnalystProfile struct with all statistics
     */
    function getAnalystProfile(address analyst) external view returns (AnalystProfile memory) {
        return analysts[analyst];
    }

    /**
     * @dev Get analyst's reputation tier
     * @param analyst Address of the analyst
     * @return Tier level (0-based)
     */
    function getAnalystTier(address analyst) external view returns (uint256) {
        uint256 reputation = analysts[analyst].reputation;
        
        // Determine tier based on reputation
        if (reputation >= 800) return 4; // Expert
        if (reputation >= 600) return 3; // Advanced
        if (reputation >= 400) return 2; // Intermediate
        if (reputation >= 200) return 1; // Beginner
        return 0; // Novice
    }

    /**
     * @dev Apply reputation decay for inactive analysts
     */
    function applyReputationDecay() external onlyRole(ADMIN_ROLE) {
        require(block.timestamp >= decayTimestamp + 30 days, "Decay not due yet");
        
        for (uint256 i = 0; i < activeAnalysts.length; i++) {
            address analyst = activeAnalysts[i];
            AnalystProfile storage profile = analysts[analyst];
            
            if (block.timestamp >= profile.lastActiveTimestamp + 30 days) {
                uint256 decayAmount = profile.reputation * DECAY_RATE / 100;
                if (profile.reputation > decayAmount) {
                    profile.reputation -= decayAmount;
                } else {
                    profile.reputation = MIN_REPUTATION;
                }
                
                emit ReputationDecayed(analyst, decayAmount);
            }
        }
        
        decayTimestamp = block.timestamp;
    }

    /**
     * @dev Initialize reputation tiers
     */
    function _initializeReputationTiers() internal {
        reputationTiers[0] = ReputationTier(0, 100, 0, "Novice");
        reputationTiers[1] = ReputationTier(200, 150, 5, "Beginner");
        reputationTiers[2] = ReputationTier(400, 200, 10, "Intermediate");
        reputationTiers[3] = ReputationTier(600, 300, 20, "Advanced");
        reputationTiers[4] = ReputationTier(800, 500, 35, "Expert");
    }

    /**
     * @dev Get reputation tier information
     * @param tier Tier level
     * @return ReputationTier struct
     */
    function getReputationTier(uint256 tier) external view returns (ReputationTier memory) {
        return reputationTiers[tier];
    }

    /**
     * @dev Get top analysts by reputation
     * @param limit Number of analysts to return
     * @return Array of analyst addresses sorted by reputation
     */
    function getTopAnalysts(uint256 limit) external view returns (address[] memory) {
        require(limit > 0 && limit <= activeAnalysts.length, "Invalid limit");
        
        // Simple selection sort for top analysts (gas-efficient for small lists)
        address[] memory sortedAnalysts = new address[](limit);
        uint256[] memory reputations = new uint256[](limit);
        
        for (uint256 i = 0; i < limit && i < activeAnalysts.length; i++) {
            address maxAnalyst = activeAnalysts[0];
            uint256 maxReputation = 0;
            uint256 maxIndex = 0;
            
            for (uint256 j = 0; j < activeAnalysts.length; j++) {
                bool alreadySelected = false;
                for (uint256 k = 0; k < i; k++) {
                    if (sortedAnalysts[k] == activeAnalysts[j]) {
                        alreadySelected = true;
                        break;
                    }
                }
                
                if (!alreadySelected && analysts[activeAnalysts[j]].reputation > maxReputation) {
                    maxAnalyst = activeAnalysts[j];
                    maxReputation = analysts[activeAnalysts[j]].reputation;
                    maxIndex = j;
                }
            }
            
            sortedAnalysts[i] = maxAnalyst;
            reputations[i] = maxReputation;
        }
        
        return sortedAnalysts;
    }

    /**
     * @dev Get total number of active analysts
     * @return Number of active analysts
     */
    function getTotalAnalysts() external view returns (uint256) {
        return totalAnalysts;
    }

    /**
     * @dev Check if analyst is eligible for specific tier benefits
     * @param analyst Address of the analyst
     * @param requiredTier Minimum required tier
     * @return Whether analyst meets tier requirement
     */
    function isEligibleForTier(address analyst, uint256 requiredTier) external view returns (bool) {
        uint256 currentTier = this.getAnalystTier(analyst);
        return currentTier >= requiredTier;
    }
}