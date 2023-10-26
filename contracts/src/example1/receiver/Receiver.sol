//SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

import {ReceiverLib} from "./ReceiverLib.sol";
import {AdminLib} from "../admin/AdminLib.sol";
import {ErrorLib} from "../error/ErrorLib.sol";

// Get the Switchboard Library - this is the Core Mainnet Deployment, you can swap this for one of the networks below
import {SwitchboardCallbackHandler} from "@switchboard-xyz/evm.js/contracts/SwitchboardCallbackHandler.sol";

contract Receiver is SwitchboardCallbackHandler {
    function callback(
        int256 data,
        uint256 timestamp
    ) external isSwitchboardCaller isFunctionId {
        // Set function id on first callback
        address functionId = getEncodedFunctionId();
        if (AdminLib.functionId() == address(0)) {
            AdminLib.setFunctionId(functionId);
        }

        ReceiverLib.callback(data, timestamp);
    }

    function viewData() external view returns (int256, uint256) {
        return ReceiverLib.viewData();
    }

    // SwitchboardCallbackHandler functions integrated below

    function getSwithboardAddress() internal view override returns (address) {
        return AdminLib.switchboard();
    }

    function getSwitchboardFunctionId()
        internal
        view
        override
        returns (address)
    {
        return AdminLib.functionId();
    }

    function getEncodedFunctionId() internal pure returns (address) {
        if (msg.data.length < 20) {
            revert SwitchboardCallbackHandler__MissingFunctionId();
        }

        address receivedFunctionId;
        assembly {
            receivedFunctionId := shr(96, calldataload(sub(calldatasize(), 20)))
        }
        return receivedFunctionId;
    }
}
