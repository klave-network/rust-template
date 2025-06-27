// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

contract CrossChainToken is ERC20, Ownable, Pausable, ReentrancyGuard {

    constructor(string memory name, string memory symbol, address initialOwner) ERC20(name, symbol) Ownable(initialOwner) {}

    function mint(address to, uint256 amount) external onlyOwner whenNotPaused nonReentrant {
        _mint(to, amount);
        emit Minted(to, amount);
    }

    function burn(address to, uint256 amount) external onlyOwner whenNotPaused nonReentrant {
        _burn(to, amount);
        emit Burned(to, amount);
    }

    function pause() external onlyOwner {
        _pause();
    }

    function unpause() external onlyOwner {
        _unpause();
    }

    event Minted(address indexed to, uint256 amount);
    event Burned(address indexed to, uint256 amount);
}
