// requires a running IK-KAH chopsticks
// for setup, refer to
// https://github.com/integritee-network/parachain/issues/323
//
// As IK sovereign, we will send a xcm to KAH to transact/execute a system.remark_with_event
// all fees will be paid in TEER and converted to KSM on KAH as needed

// `pah` and 'kah' are the names we gave to `bun papi add`.
import {
    itk, // bun papi add itk -w http://localhost:8001
    kah, // bun papi add kah -w http://localhost:8000
    DispatchRawOrigin,
    XcmV5Junction,
    XcmV5Junctions,
    XcmV5NetworkId,
    XcmV3MultiassetFungibility,
    XcmV5Instruction,
    XcmVersionedAssetId,
    XcmVersionedLocation,
    XcmVersionedXcm,
    XcmV2OriginKind,
    XcmV5AssetFilter, XcmV5WildAsset
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
import {take} from "rxjs"

// Useful constants.
const KAH_PARA_ID = 1000;
const IK_PARA_ID = 2015;

// We're running against chopsticks with wasm-override to get XCMv5 support.
// `npx @acala-network/chopsticks@latest xcm --p=kusama-asset-hub --p=./configs/integritee-kusama.yml`
const KAH_WS_URL = "ws://localhost:8000";
const IK_WS_URL = "ws://localhost:8001";

// if running against the bridge zombienet instead, use these:
//const KAH_WS_URL = "ws://localhost:9010";
//const IK_WS_URL = "ws://localhost:9144";

const IK_FROM_KAH = {
    parents: 1,
    interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(IK_PARA_ID)),
};
const KAH_FROM_IK = {
    parents: 1,
    interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(KAH_PARA_ID)),
};
// XCM.
const XCM_VERSION = 5;

const TEER_UNITS = 1_000_000_000_000n;

const KSM_FROM_KUSAMA_PARACHAINS = {
    parents: 1,
    interior: XcmV5Junctions.Here(),
};
const TEER_FROM_SELF = {
    parents: 0,
    interior: XcmV5Junctions.Here(),
};
const TEER_FROM_SIBLING = {
    parents: 1,
    interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(2015)),
};

// Setup clients...
const kahClient = createClient(
    withPolkadotSdkCompat(getWsProvider(KAH_WS_URL)),
);
const kahApi = kahClient.getTypedApi(kah);

const itkClient = createClient(
    withPolkadotSdkCompat(getWsProvider(IK_WS_URL)),
);
const itkApi = itkClient.getTypedApi(itk);

// The whole execution of the script.
main();

// We'll teleport KSM from Asset Hub to People.
// Using the XcmPaymentApi and DryRunApi, we'll estimate the XCM fees accurately.
async function main() {
    // The amount of TEER we wish to teleport besides paying fees.
    const transferAmount = 0n;
    // We overestimate both local and remote fees, these will be adjusted by the dry run below.
    const localFeesHighEstimate = 1n * TEER_UNITS / 10n;
    const remoteFeesHighEstimate = 2n * TEER_UNITS;

    const stx = await itkApi.tx.System.remark_with_event({remark: Binary.fromText("Let's trigger state migration")})
    const signer = getAliceSigner();
    await stx.signAndSubmit(signer);
    console.log("triggered state migrations if necessary. waiting for a bit....")
    // wait for chopsticks api's to catch up
    await new Promise(resolve => setTimeout(resolve, 5000));

    // We create a tentative XCM, one with the high estimates for fees.
    const tentativeXcm = await createXcm(
        transferAmount,
        localFeesHighEstimate,
        remoteFeesHighEstimate,
    );
    console.dir(stringify(tentativeXcm));

    // We can now query the root account on the source chain.
    const rootAccountLocal = await itkApi.apis.LocationToAccountApi.convert_location(XcmVersionedLocation.V5(TEER_FROM_SELF))
    if (!rootAccountLocal.success) {
        console.error("Failed to get root account on source chain: ", rootAccountLocal);
        await itkClient.destroy();
        await kahClient.destroy();
        return;
    }
    const rootAccountInfo = await itkApi.query.System.Account.getValue(rootAccountLocal.value)
    console.log("Root account on source chain: ", rootAccountLocal, " with AccountInfo ", rootAccountInfo);

    const weightRes = await itkApi.apis.XcmPaymentApi.query_xcm_weight(tentativeXcm);
    if (!weightRes.success) {
        console.error("Failed to get weight for tentative XCM: ", weightRes);
        await itkClient.destroy();
        await kahClient.destroy();
        return;
    }
    console.log(weightRes);
    const tentativeTx = itkApi.tx.PolkadotXcm.execute({
        message: tentativeXcm,
        max_weight: weightRes.value, // Arbitrary weight, we will adjust it later.
    });
    console.log(tentativeTx)
    const tentativeTxSudo = itkApi.tx.Sudo.sudo({call: tentativeTx.decodedCall});
    console.log("encoded tentative call on source chain (e.g. to try with chopsticks): ", (await tentativeTxSudo.getEncodedData()).asHex());

    // This will give us the adjusted estimates, much more accurate than before.
    const [localFeesEstimate, remoteFeesEstimate] =
        (await estimateFees(tentativeXcm))!;

    // With these estimates, we can create the final XCM to execute.
    const xcm = await createXcm(
        transferAmount,
        localFeesEstimate,
        remoteFeesHighEstimate, // TODO: account for conversion from TEER to DOT here when using the updated estimate
    );
    console.log("Executing XCM now....")
    const weightResult = await itkApi.apis.XcmPaymentApi.query_xcm_weight(xcm);
    if (weightResult.success) {
        const tx = itkApi.tx.PolkadotXcm.execute({
            message: xcm,
            max_weight: weightResult.value,
        });
        const stx = await itkApi.tx.Sudo.sudo({call: tx.decodedCall})
        const signer = getAliceSigner();
        const result = await stx.signAndSubmit(signer);
        console.dir(stringify(result.txHash));
        console.log("Await System.Remarked event on destination chain...")
        await kahApi.event.System.Remarked.watch()
            .pipe(take(1))
            .forEach((event) => {
                console.log("Event received: System.Remarked from ", event.payload.sender, " with hash ", event.payload.hash.asHex());
            })
    }
    await itkClient.destroy();
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

async function createXcm(
    transferAmount: bigint,
    localFees: bigint,
    remoteFees: bigint,
): Promise<XcmVersionedXcm> {
    const executeOnPah = kahApi.tx.System.remark_with_event({remark: Binary.fromText("Hello Polkadot")})
    const teerToWithdraw = {
        id: TEER_FROM_SELF,
        fun: XcmV3MultiassetFungibility.Fungible(
            transferAmount + localFees + remoteFees,
        ),
    };
    const teerForRemoteFees = {
        id: TEER_FROM_SELF,
        fun: XcmV3MultiassetFungibility.Fungible(remoteFees),
    };
    const ksmForRemoteFees = {
        id: KSM_FROM_KUSAMA_PARACHAINS,
        fun: XcmV3MultiassetFungibility.Fungible(remoteFees),
    };
    const teerForRemoteFilter = XcmV5AssetFilter.Definite([teerForRemoteFees]);

    const xcm = XcmVersionedXcm.V5([
        // we're root on source, so no fees must be paid.
        // Still, we need to withdraw an asset which can pay fees on destination
        XcmV5Instruction.WithdrawAsset([teerToWithdraw]),
        XcmV5Instruction.SetAppendix([
            XcmV5Instruction.RefundSurplus(),
            XcmV5Instruction.DepositAsset({
                assets: XcmV5AssetFilter.Wild(XcmV5WildAsset.All()),
                beneficiary: TEER_FROM_SELF,
            })
        ]),
        XcmV5Instruction.InitiateTransfer({
            destination: KAH_FROM_IK,
            preserve_origin: true,
            remote_fees: Enum("Teleport", teerForRemoteFilter),
            assets: [],
            remote_xcm: [
                XcmV5Instruction.SetAppendix([
                    XcmV5Instruction.RefundSurplus(),
                    XcmV5Instruction.DepositAsset({
                        assets: XcmV5AssetFilter.Wild(XcmV5WildAsset.All()),
                        beneficiary: TEER_FROM_SIBLING,
                    })
                ]),
                XcmV5Instruction.Transact({
                    origin_kind: XcmV2OriginKind.SovereignAccount(),
                    call: await executeOnPah.getEncodedData(),
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
    const xcmWeight = await itkApi.apis.XcmPaymentApi.query_xcm_weight(xcm);
    if (!xcmWeight.success) {
        console.error("xcmWeight failed: ", xcmWeight);
        return;
    }
    console.log("xcmWeight: ", xcmWeight.value);

    // Execution fees are purely a function of the weight.
    const executionFees =
        await itkApi.apis.XcmPaymentApi.query_weight_to_asset_fee(
            xcmWeight.value,
            XcmVersionedAssetId.V5(TEER_FROM_SELF),
        );
    if (!executionFees.success) {
        console.error("executionFees failed: ", executionFees);
        return;
    }
    console.log("executionFees: ", executionFees.value);

    const tx = itkApi.tx.PolkadotXcm.execute({
        message: xcm,
        max_weight: xcmWeight.value,
    });

    const dryRunResult = await itkApi.apis.DryRunApi.dry_run_call(
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
        XcmVersionedLocation.V5(IK_FROM_KAH),
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
    const remoteDryRunResult = await kahApi.apis.DryRunApi.dry_run_xcm(
        XcmVersionedLocation.V5({
            parents: 1,
            interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(IK_PARA_ID)),
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
        await itkApi.apis.XcmPaymentApi.query_xcm_weight(messageToPah);
    if (!remoteWeight.success) {
        console.error("remoteWeight failed: ", remoteWeight);
        return;
    }
    console.log("remoteWeight: ", remoteWeight.value);

    // Remote fees are only execution.
    const remoteFeesInDot =
        await itkApi.apis.XcmPaymentApi.query_weight_to_asset_fee(
            remoteWeight.value,
            XcmVersionedAssetId.V5(KSM_FROM_KUSAMA_PARACHAINS),
        );

    if (!remoteFeesInDot.success) {
        console.error("remoteFeesInDot failed: ", remoteFeesInDot);
        return;
    }
    console.log("remoteFeesInDot: ", remoteFeesInDot);
    const remoteFeesInTeer = remoteFeesInDot.value * 100n;
    console.log("remoteFeesInTeer: ", remoteFeesInTeer);
    return [localFees, remoteFeesInTeer];
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