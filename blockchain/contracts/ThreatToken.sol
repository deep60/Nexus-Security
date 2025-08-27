pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Pausable.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

/**
 * @title ThreatToken
 * @dev ERC20 token for Nexus-Security threat intelligence marketplace
 * 
 * Features:
 * - Standard ERC20 functionality
 * - Burnable tokens for deflationary mechanics
 * - Pausable for emergency stops
 * - Role-based access control
 * - Staking mechanism for analysis engines
 * - Reward distribution for accurate threat detection
 * - Anti-manipulation safeguards
 */

 contract ThreatToken is ERC20, ERC20Burnable, ERC20Pausable, AccessControl,