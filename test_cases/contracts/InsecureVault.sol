// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

contract InsecureVault {
    address public owner;

    constructor() {
        owner = msg.sender;
    }

    function privileged(address target, bytes calldata data) external {
        require(tx.origin == owner, "owner only");
        (bool ok,) = target.delegatecall(data);
        require(ok, "delegatecall failed");
    }

    function send(address payable target, uint256 value) external {
        (bool ok,) = target.call{value: value}("");
        require(ok, "call failed");
    }
}
