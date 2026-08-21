// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

contract SafeCounter {
    uint256 public value;

    function increment() external {
        value += 1;
    }
}
