// reuires a running bridge-zombienet. See ../bridges
//
// As KAH sovereign, we will send a xcm to PAH to transact/execute a system.remark
// .

// `pah` and 'kah' are the names we gave to `bun papi add`.
import {
    pah,
    kah,
    DispatchRawOrigin,
    XcmV5Junction,
    XcmV5Junctions,
    XcmV5NetworkId,
    XcmV3MultiassetFungibility,
    XcmV3WeightLimit,
    XcmV5AssetFilter,
    XcmV2OriginKind,
    XcmV5WildAsset,
    XcmV5Instruction,
    XcmVersionedAssetId,
    XcmVersionedLocation,
    XcmVersionedXcm,
} from "@polkadot-api/descriptors";
import {
    createClient,
    Enum,
    Binary,
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
const KAH_PARA_ID = 1000;
const PAH_PARA_ID = 1000;

// We're using localhost here since this was tested with chopsticks.
// For production, replace //Alice with a real account and use a public rpc, for example: "wss://polkadot-people-rpc.polkadot.io".
const KAH_WS_URL = "ws://localhost:9010";
const PAH_WS_URL = "ws://localhost:9910";
// How to get to People from the perspective of Asset Hub.

const PAH_FROM_KAH = {
    parents: 2,
    interior: XcmV5Junctions.X2([XcmV5Junction.GlobalConsensus(XcmV5NetworkId.Polkadot()), XcmV5Junction.Parachain(PAH_PARA_ID)]),
};
const KAH_FROM_PAH = {
    parents: 2,
    interior: XcmV5Junctions.X2([XcmV5Junction.GlobalConsensus(XcmV5NetworkId.Kusama()), XcmV5Junction.Parachain(KAH_PARA_ID)]),
};
// XCM.
const XCM_VERSION = 5;
// DOT.
const KSM_UNITS = 1_000_000_000_000n;

const KSM_FROM_KUSAMA_PARACHAINS = {
    parents: 1,
    interior: XcmV5Junctions.Here(),
};
const DOT_FROM_POLKADOT_PARACHAINS = {
    parents: 1,
    interior: XcmV5Junctions.Here(),
};
const KSM_FROM_POLKADOT_PARACHAINS = {
    parents: 2,
    interior: XcmV5Junctions.X1(XcmV5Junction.GlobalConsensus(XcmV5NetworkId.Kusama())),
};

// Setup clients...
const kahClient = createClient(
    withPolkadotSdkCompat(getWsProvider(KAH_WS_URL)),
);
const kahApi = kahClient.getTypedApi(kah);

const pahClient = createClient(
    withPolkadotSdkCompat(getWsProvider(PAH_WS_URL)),
);
const pahApi = pahClient.getTypedApi(pah);

// The whole execution of the script.
await main();

// We'll teleport KSM from Asset Hub to People.
// Using the XcmPaymentApi and DryRunApi, we'll estimate the XCM fees accurately.
async function main() {
    // The amount of KSM we wish to teleport.
    const transferAmount = 10n * KSM_UNITS;
    // We overestimate both local and remote fees, these will be adjusted by the dry run below.
    const localFeesHighEstimate = 1n * KSM_UNITS;
    const remoteFeesHighEstimate = 1n * KSM_UNITS;
    // We create a tentative XCM, one with the high estimates for fees.
    const tentativeXcm = await createBridgeTransfer(
        transferAmount,
        localFeesHighEstimate,
        remoteFeesHighEstimate,
    );
    console.dir(stringify(tentativeXcm));

    // This will give us the adjusted estimates, much more accurate than before.
    const [localFeesEstimate, remoteFeesEstimate] =
        (await estimateFees(tentativeXcm))!;

    // With these estimates, we can create the final XCM to execute.
    const xcm = await createBridgeTransfer(
        transferAmount,
        localFeesEstimate,
        remoteFeesEstimate,
    );
    const weightResult = await pahApi.apis.XcmPaymentApi.query_xcm_weight(xcm);

    // We get the weight and we execute.
    if (weightResult.success) {
        console.log("Final XCM (won't be executed)", xcm)
    }
    await pahClient.destroy();
    await kahClient.destroy();
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

// Creates an XCM that will bridge KSM from KAH to PAH.
//
// Takes in the amount of KSM wanting to be transferred, as well as the
// amount of KSM willing to be used for local and remote fees.
async function createBridgeTransfer(
    transferAmount: bigint,
    localFees: bigint,
    remoteFees: bigint,
): Promise<XcmVersionedXcm> {
    const executeOnPah = pahApi.tx.System.remark({remark: Binary.fromText("Hello Polkadot")})
    const ksmToWithdraw = {
        id: KSM_FROM_KUSAMA_PARACHAINS,
        fun: XcmV3MultiassetFungibility.Fungible(
            transferAmount + localFees + remoteFees,
        ),
    };
    const ksmForLocalFees = {
        id: KSM_FROM_KUSAMA_PARACHAINS,
        fun: XcmV3MultiassetFungibility.Fungible(localFees),
    };
    const dotForRemoteFees = {
        id: DOT_FROM_POLKADOT_PARACHAINS,
        fun: XcmV3MultiassetFungibility.Fungible(remoteFees),
    };
    const xcm = XcmVersionedXcm.V5([
        XcmV5Instruction.WithdrawAsset([ksmToWithdraw]),
        XcmV5Instruction.PayFees({
            asset: ksmForLocalFees,
        }),
        XcmV5Instruction.InitiateTransfer({
            destination: PAH_FROM_KAH,
            preserve_origin: true,
            assets: [],
            remote_xcm: [
                XcmV5Instruction.Transact({
                    origin_kind: XcmV2OriginKind.SovereignAccount(),
                    call: await executeOnPah.getEncodedData(),
                }),
                XcmV5Instruction.RefundSurplus(),
            ],
        }),
        XcmV5Instruction.RefundSurplus(),
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
    const xcmWeight = await pahApi.apis.XcmPaymentApi.query_xcm_weight(xcm);
    if (!xcmWeight.success) {
        console.error("xcmWeight failed: ", xcmWeight);
        return;
    }
    console.log("xcmWeight: ", xcmWeight.value);

    // Execution fees are purely a function of the weight.
    const executionFees =
        await pahApi.apis.XcmPaymentApi.query_weight_to_asset_fee(
            xcmWeight.value,
            XcmVersionedAssetId.V5(KSM_FROM_KUSAMA_PARACHAINS),
        );
    if (!executionFees.success) {
        console.error("executionFees failed: ", executionFees);
        return;
    }
    console.log("executionFees: ", executionFees.value);

    const tx = pahApi.tx.PolkadotXcm.execute({
        message: xcm,
        max_weight: xcmWeight.value,
    });

    const dryRunResult = await pahApi.apis.DryRunApi.dry_run_call(
        Enum("system", DispatchRawOrigin.Root()),
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
            location.type === "V5" &&
            location.value.parents === 1 &&
            location.value.interior.type === "X1" &&
            location.value.interior.value.type === "Parachain" &&
            location.value.interior.value.value === KAH_PARA_ID,
    )!;
    // Found it.
    const messageToPah = messages[0];

    // We're only dealing with version 4.
    if (messageToPah?.type !== "V5") {
        console.error("messageToPah failed: expected XCMv5");
        return;
    }

    // We get the delivery fees using the size of the forwarded xcm.
    const deliveryFees = await kahApi.apis.XcmPaymentApi.query_delivery_fees(
        XcmVersionedLocation.V5(PAH_FROM_KAH),
        messageToPah,
    );
    // Fees should be of the version we expect and fungible tokens, in particular, KSM.
    if (
        !deliveryFees.success ||
        deliveryFees.value.type !== "V5" ||
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
    const remoteDryRunResult = await pahApi.apis.DryRunApi.dry_run_xcm(
        XcmVersionedLocation.V5({
            parents: 1,
            interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(PAH_PARA_ID)),
        }),
        messageToPah,
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
        await pahApi.apis.XcmPaymentApi.query_xcm_weight(messageToPah);
    if (!remoteWeight.success) {
        console.error("remoteWeight failed: ", remoteWeight);
        return;
    }
    console.log("remoteWeight: ", remoteWeight.value);

    // Remote fees are only execution.
    const remoteFeesInDot =
        await pahApi.apis.XcmPaymentApi.query_weight_to_asset_fee(
            remoteWeight.value,
            XcmVersionedAssetId.V5(KSM_FROM_KUSAMA_PARACHAINS),
        );

    if (!remoteFeesInDot.success) {
        console.error("remoteFeesInDot failed: ", remoteFeesInDot);
        return;
    }
    console.log("remoteFeesInDot: ", remoteFeesInDot);
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