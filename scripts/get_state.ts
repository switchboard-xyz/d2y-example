import { ethers } from "hardhat";
import moment from "moment-timezone";
import Big from "big.js";
const BigNumber = require("bignumber.js");

async function main() {
  const sbPushAddress = process.env.EXAMPLE_PROGRAM ?? "";

  const divisor = new BigNumber("100000000");

  if (!sbPushAddress) {
    throw new Error(
      "Please set the diamond address with: export EXAMPLE_PROGRAM=..."
    );
  }

  const push = await ethers.getContractAt("Receiver", sbPushAddress);
  const p = await push.deployed();

  const [iv, timestamp] = await p.viewData();
  console.log("============");
  console.log(`Implied Vol: ${new Big(iv.toString()).div(divisor).toString()}`);
  console.log(`Timestamp: ${timestamp}`);
  console.log("============");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
