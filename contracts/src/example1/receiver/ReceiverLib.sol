//SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

library ReceiverLib {
    bytes32 constant DIAMOND_STORAGE_POSITION = keccak256("receiverlib.v1.storage");

    struct DiamondStorage {
        uint256 latestTimestamp;
        int256 latestData;
    }

    function diamondStorage()
        internal
        pure
        returns (DiamondStorage storage ds)
    {
        bytes32 position = DIAMOND_STORAGE_POSITION;
        assembly {
            ds.slot := position
        }
    }

    // Switchboard Function will call this function with the feed ids and values
    function callback(
        int256 data,
        uint256 timestamp // time the query was started
    ) internal {
        DiamondStorage storage ds = diamondStorage();
        ds.latestTimestamp = timestamp;
        ds.latestData = data;
    }

    function viewData() internal view returns (int256 data, uint256 timestamp) {
        DiamondStorage storage ds = diamondStorage();
        data = ds.latestData;
        timestamp = ds.latestTimestamp;
    }
}
