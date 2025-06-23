// Example by Francisco Aguirre
//
// We'll teleport DOT from Asset Hub to People using XCMv4
// .
// https://hackmd.io/@n9QBuDYOQXG-nWCBrwx8YQ/rkRNb5m71e
// https://gist.github.com/franciscoaguirre/c1b2a9480744bbe698bfd74f9a0c0e26

// `pah` and 'ppeople' are the names we gave to `bun papi add`.
import {
    pah,
    DispatchRawOrigin,
    ppeople,
    XcmV3Junction,
    XcmV3Junctions,
    XcmV3MultiassetFungibility,
    XcmV3WeightLimit,
    XcmV4AssetAssetFilter,
    XcmV4AssetWildAsset,
    XcmV4Instruction,
    XcmVersionedAssetId,
    XcmVersionedLocation,
    XcmVersionedXcm,
} from "@polkadot-api/descriptors";
import {
    createClient,
    Enum,
    FixedSizeBinary,
    type PolkadotSigner,
} from "polkadot-api";
// import from "polkadot-api/ws-provider/node"
// if you are running in a NodeJS environment
import {getWsProvider} from "polkadot-api/ws-provider/node";
import {withPolkadotSdkCompat} from "polkadot-api/polkadot-sdk-compat";
import {getPolkadotSigner} from "polkadot-api/signer";
import {
    DEV_PHRASE,
    entropyToMiniSecret,
    mnemonicToEntropy,
} from "@polkadot-labs/hdkd-helpers";
import {sr25519CreateDerive} from "@polkadot-labs/hdkd";

// Useful constants.
// People.
const PEOPLE_PARA_ID = 1004;
// We're using localhost here since this was tested with chopsticks.
// For production, replace //Alice with a real account and use a public rpc, for example: "wss://polkadot-people-rpc.polkadot.io".
const PEOPLE_WS_URL = "ws://localhost:8001";
// Asset Hub.
const ASSET_HUB_PARA_ID = 1000;
// We're using localhost here since this was tested with chopsticks.
// For production, replace //Alice with a real account and use a public rpc, for example: "wss://polkadot-asset-hub-rpc.polkadot.io".
const ASSET_HUB_WS_URL = "ws://localhost:8000";
// How to get to People from the perspective of Asset Hub.
const PEOPLE_FROM_AH = {
    parents: 1,
    interior: XcmV3Junctions.X1(XcmV3Junction.Parachain(PEOPLE_PARA_ID)),
};
// XCM.
const XCM_VERSION = 4;
// DOT.
const DOT_UNITS = 10_000_000_000n;
const DOT_FROM_PARACHAINS = {
    parents: 1,
    interior: XcmV3Junctions.Here(),
};
// Alice's SS58 account for Polkadot.
const ACCOUNT = "15oF4uVJwmo4TdGW7VfQxNLavjCXviqxT9S1MgbjMNHr6Sp5";

// Setup client...
const ahClient = createClient(
    withPolkadotSdkCompat(getWsProvider(ASSET_HUB_WS_URL)),
);

// ...and typed api.
const ahApi = ahClient.getTypedApi(pah);

// The whole execution of the script.
await main();

// We'll teleport DOT from Asset Hub to People.
// Using the XcmPaymentApi and DryRunApi, we'll estimate the XCM fees accurately.
async function main() {
    // The amount of DOT we wish to teleport.
    const transferAmount = 10n * DOT_UNITS;
    // We overestimate both local and remote fees, these will be adjusted by the dry run below.
    const localFeesHighEstimate = 1n * DOT_UNITS;
    const remoteFeesHighEstimate = 1n * DOT_UNITS;
    // We create a tentative XCM, one with the high estimates for fees.
    const tentativeXcm = createTeleport(
        transferAmount,
        localFeesHighEstimate,
        remoteFeesHighEstimate,
    );
    console.dir(stringify(tentativeXcm));

    // This will give us the adjusted estimates, much more accurate than before.
    const [localFeesEstimate, remoteFeesEstimate] =
        (await estimateFees(tentativeXcm))!;

    // With these estimates, we can create the final XCM to execute.
    const xcm = createTeleport(
        transferAmount,
        localFeesEstimate,
        remoteFeesEstimate,
    );

    // We get the weight and we execute.
    console.log("Executing XCM now....")
    const weightResult = await ahApi.apis.XcmPaymentApi.query_xcm_weight(xcm);
    if (weightResult.success) {
        const tx = ahApi.tx.PolkadotXcm.execute({
            message: xcm,
            max_weight: weightResult.value,
        });
        const signer = getAliceSigner();
        const result = await tx.signAndSubmit(signer);
        console.dir(stringify(result));
    }
    await ahClient.destroy();
}

// Helper function to convert bigints to strings and binaries to hex strings in objects.
function converter(_: string, value: any): string {
    if (typeof value === "bigint") {
        return value.toString();
    } else if (typeof value === "object" && value.asHex !== undefined) {
        return value.asHex();
    } else {
        return value;
    }
}

// Helper function to stringify an object using `converter` to also show bigints and binaries.
function stringify(obj: any): string {
    return JSON.stringify(obj, converter, 2);
}

// Creates an XCM that will teleport DOT from Asset Hub to People.
//
// Takes in the amount of DOT wanting to be transferred, as well as the
// amount of DOT willing to be used for local and remote fees.
function createTeleport(
    transferAmount: bigint,
    localFees: bigint,
    remoteFees: bigint,
): XcmVersionedXcm {
    const beneficiary = {
        parents: 0,
        interior: XcmV3Junctions.X1(
            XcmV3Junction.AccountId32({
                id: FixedSizeBinary.fromAccountId32(ACCOUNT),
            }),
        ),
    };
    const dotToWithdraw = {
        id: DOT_FROM_PARACHAINS,
        fun: XcmV3MultiassetFungibility.Fungible(
            transferAmount + localFees + remoteFees,
        ),
    };
    const dotForLocalFees = {
        id: DOT_FROM_PARACHAINS,
        fun: XcmV3MultiassetFungibility.Fungible(localFees),
    };
    const dotForRemoteFees = {
        id: DOT_FROM_PARACHAINS,
        fun: XcmV3MultiassetFungibility.Fungible(remoteFees),
    };
    const xcm = XcmVersionedXcm.V4([
        XcmV4Instruction.WithdrawAsset([dotToWithdraw]),
        XcmV4Instruction.BuyExecution({
            fees: dotForLocalFees,
            // We allow maximum weight bought with the specified fees.
            weight_limit: XcmV3WeightLimit.Unlimited(),
        }),
        XcmV4Instruction.InitiateTeleport({
            dest: PEOPLE_FROM_AH,
            assets: XcmV4AssetAssetFilter.Wild(XcmV4AssetWildAsset.AllCounted(1)),
            xcm: [
                XcmV4Instruction.BuyExecution({
                    fees: dotForRemoteFees,
                    weight_limit: XcmV3WeightLimit.Unlimited(),
                }),
                XcmV4Instruction.DepositAsset({
                    assets: XcmV4AssetAssetFilter.Wild(XcmV4AssetWildAsset.AllCounted(1)),
                    beneficiary,
                }),
            ],
        }),
    ]);

    return xcm;
}

// Estimates both local and remote fees for a given XCM.
//
// This is the mechanism showcased on this script.
// Uses the XcmPaymentApi to get local fees, both execution and delivery.
// Then uses the DryRunApi to get the sent XCM and estimates remote fees
// connecting to the destination chain.
//
// If there's any issue and fees couldn't be estimated, returns undefined.
async function estimateFees(
    xcm: XcmVersionedXcm,
): Promise<[bigint, bigint] | undefined> {
    const xcmWeight = await ahApi.apis.XcmPaymentApi.query_xcm_weight(xcm);
    if (!xcmWeight.success) {
        console.error("xcmWeight failed: ", xcmWeight);
        return;
    }
    console.log("xcmWeight: ", xcmWeight.value);

    // Execution fees are purely a function of the weight.
    const executionFees =
        await ahApi.apis.XcmPaymentApi.query_weight_to_asset_fee(
            xcmWeight.value,
            XcmVersionedAssetId.V4(DOT_FROM_PARACHAINS),
        );
    if (!executionFees.success) {
        console.error("executionFees failed: ", executionFees);
        return;
    }
    console.log("executionFees: ", executionFees.value);

    const tx = ahApi.tx.PolkadotXcm.execute({
        message: xcm,
        max_weight: xcmWeight.value,
    });

    const dryRunResult = await ahApi.apis.DryRunApi.dry_run_call(
        Enum("system", DispatchRawOrigin.Signed(ACCOUNT)),
        tx.decodedCall,
        XCM_VERSION,
    );
    if (!dryRunResult.success || !dryRunResult.value.execution_result.success) {
        console.error("dryRunResult failed: ", dryRunResult);
        return;
    }
    console.log("dryRunResult: ", dryRunResult.value);

    // XCM execution might result in multiple messages being sent.
    // That's why we need to search for our message in the `forwarded_xcms` array.
    const [_, messages] = dryRunResult.value.forwarded_xcms.find(
        ([location, _]) =>
            location.type === "V4" &&
            location.value.parents === 1 &&
            location.value.interior.type === "X1" &&
            location.value.interior.value.type === "Parachain" &&
            location.value.interior.value.value === PEOPLE_PARA_ID,
    )!;
    // Found it.
    const messageToPeople = messages[0];
    // Now that we know the XCM that will be executed on the people chain,
    // we need to connect to it so we can estimate the fees.
    const peopleClient = createClient(
        withPolkadotSdkCompat(getWsProvider(PEOPLE_WS_URL)),
    );
    const peopleApi = peopleClient.getTypedApi(ppeople);

    // We're only dealing with version 4.
    if (messageToPeople?.type !== "V4") {
        console.error("messageToPeople failed: expected XCMv4");
        return;
    }

    // We get the delivery fees using the size of the forwarded xcm.
    const deliveryFees = await ahApi.apis.XcmPaymentApi.query_delivery_fees(
        XcmVersionedLocation.V4(PEOPLE_FROM_AH),
        messageToPeople,
    );
    // Fees should be of the version we expect and fungible tokens, in particular, DOT.
    if (
        !deliveryFees.success ||
        deliveryFees.value.type !== "V4" ||
        deliveryFees.value.value.length < 1 ||
        deliveryFees.value.value[0]?.fun?.type !== "Fungible"
    ) {
        console.error("deliveryFees failed: ", deliveryFees);
        return;
    }
    console.log("deliveryFees: ", deliveryFees.value.value);

    // Local fees are execution + delivery.
    const localFees = executionFees.value + deliveryFees.value.value[0].fun.value;

    // Now we dry run on the destination.
    const remoteDryRunResult = await peopleApi.apis.DryRunApi.dry_run_xcm(
        XcmVersionedLocation.V4({
            parents: 1,
            interior: XcmV3Junctions.X1(XcmV3Junction.Parachain(ASSET_HUB_PARA_ID)),
        }),
        messageToPeople,
    );
    if (
        !remoteDryRunResult.success ||
        remoteDryRunResult.value.execution_result.type !== "Complete"
    ) {
        console.error("remoteDryRunResult failed: ", remoteDryRunResult);
        return;
    }
    console.log("remoteDryRunResult: ", remoteDryRunResult.value);

    const remoteWeight =
        await peopleApi.apis.XcmPaymentApi.query_xcm_weight(messageToPeople);
    if (!remoteWeight.success) {
        console.error("remoteWeight failed: ", remoteWeight);
        return;
    }
    console.log("remoteWeight: ", remoteWeight.value);

    // Remote fees are only execution.
    const remoteFeesInDot =
        await peopleApi.apis.XcmPaymentApi.query_weight_to_asset_fee(
            remoteWeight.value,
            XcmVersionedAssetId.V4(DOT_FROM_PARACHAINS),
        );

    if (!remoteFeesInDot.success) {
        console.error("remoteFeesInDot failed: ", remoteFeesInDot);
        return;
    }
    console.log("remoteFeesInDot: ", remoteFeesInDot);
    peopleClient.destroy()
    return [localFees, remoteFeesInDot.value];
}

// Just a helper function to get a signer for ALICE.
function getAliceSigner(): PolkadotSigner {
    const entropy = mnemonicToEntropy(DEV_PHRASE);
    const miniSecret = entropyToMiniSecret(entropy);
    const derive = sr25519CreateDerive(miniSecret);
    const hdkdKeyPair = derive("//Alice");
    const aliceSigner = getPolkadotSigner(
        hdkdKeyPair.publicKey,
        "Sr25519",
        hdkdKeyPair.sign,
    );
    return aliceSigner;
}