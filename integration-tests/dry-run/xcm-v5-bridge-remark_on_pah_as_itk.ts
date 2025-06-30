// requires a running IK-KAH chopsticks
// for setup, refer to
// https://github.com/integritee-network/parachain/issues/323
//
// Kusama and Polkadot side must be run separately:
// npx @acala-network/chopsticks@latest xcm --p=./configs/kusama-asset-hub.yml --p=./configs/integritee-kusama.yml
// should be ports 8000 and 8001 respectively.
// npx @acala-network/chopsticks@latest xcm --p=./configs/polkadot-asset-hub.yml --p=./configs/integritee-polkadot.yml
// should be ports 8002 and 8003 respectively.
//
// As IK sovereign, we will send a xcm to KAH to transact/execute a system.remark_with_event
// all fees will be paid in TEER and converted to KSM on KAH as needed

// `pah` and 'kah' are the names we gave to `bun papi add`.
import {
    itk, // bun papi add itk -w http://localhost:8001
    kah, // bun papi add kah -w http://localhost:8000
    pah, // bun papi add pah -w http://localhost:8002
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
const PAH_PARA_ID = 1000;
const IK_PARA_ID = 2015;

// We're running against chopsticks with wasm-override to get XCMv5 support.
// `npx @acala-network/chopsticks@latest xcm --p=kusama-asset-hub --p=./configs/integritee-kusama.yml`
const KAH_WS_URL = "ws://localhost:8000";
const IK_WS_URL = "ws://localhost:8001";
const PAH_WS_URL = "ws://localhost:8002";

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

const TEER_UNITS = 1_000_000_000_000n;
const KSM_UNITS = 1_000_000_000_000n;

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
const TEER_FROM_COUSIN = {
    parents: 2,
    interior: XcmV5Junctions.X2([XcmV5Junction.GlobalConsensus(XcmV5NetworkId.Kusama()), XcmV5Junction.Parachain(IK_PARA_ID)]),
};

// Setup clients...
const pahClient = createClient(
    withPolkadotSdkCompat(getWsProvider(PAH_WS_URL)),
);
const pahApi = pahClient.getTypedApi(pah);

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
    const localFeesHighEstimate = 0n; // we're root locally and don't pay fees, so this is just a placeholder.
    const remote1FeesHighEstimateTeer = 10n * TEER_UNITS;
    const remote2FeesHighEstimateKsm = 1n * KSM_UNITS / 10n;

    const stx = await itkApi.tx.System.remark_with_event({remark: Binary.fromText("Let's trigger state migration")})
    const signer = getAliceSigner();
    await stx.signAndSubmit(signer);
    console.log("triggered state migrations if necessary. waiting for a bit....")
    // wait for chopsticks api's to catch up
    await new Promise(resolve => setTimeout(resolve, 5000));

    const remote2FeesHighEstimateTeerConverted = await kahApi.apis.AssetConversionApi.quote_price_tokens_for_exact_tokens(TEER_FROM_SIBLING, KSM_FROM_KUSAMA_PARACHAINS, remote2FeesHighEstimateKsm, true);
    const teerPerKsm = Number(remote2FeesHighEstimateTeerConverted) / Number(remote2FeesHighEstimateKsm)
    console.log("Current AssetConversion quote for remote account: out: ", remote2FeesHighEstimateTeerConverted, " in ", remote2FeesHighEstimateKsm, " TEER. price: ", teerPerKsm, " TEER per KSM");

    //const remote2FeesHighEstimateKsmConverted = await pahApi.apis.AssetConversionApi.quote_price_tokens_for_exact_tokens(KSM_FROM_POLKADOT_PARACHAINS, DOT_FROM_SIBLING_PARACHAINS, remote2FeesHighEstimateKsm, true);
    //const teerPerKsm = Number(remote2FeesHighEstimateKsmConverted) / Number(remote2FeesHighEstimateKsm)
    //console.log("Current AssetConversion quote for remote account: out: ", remote2FeesHighEstimateKsmConverted, " in ", remote2FeesHighEstimateKsm, " TEER. price: ", teerPerKsm, " TEER per KSM");

    // We create a tentative XCM, one with the high estimates for fees.
    const tentativeXcm = await createXcm(
        transferAmount,
        localFeesHighEstimate,
        remote1FeesHighEstimateTeer,
        BigInt(Math.round(Number(remote2FeesHighEstimateTeerConverted) * 1.5)),
        remote2FeesHighEstimateKsm
    );
    console.dir(stringify(tentativeXcm));

    // We can now query the root account on the source chain.
    const rootAccountLocal = await itkApi.apis.LocationToAccountApi.convert_location(XcmVersionedLocation.V5(TEER_FROM_SELF))
    if (!rootAccountLocal.success) {
        console.error("Failed to get root account on source chain: ", rootAccountLocal);
        await itkClient.destroy();
        await kahClient.destroy();
        await pahClient.destroy();
        return;
    }
    const rootAccountInfo = await itkApi.query.System.Account.getValue(rootAccountLocal.value)
    console.log("Root account on source chain: ", rootAccountLocal, " with AccountInfo ", rootAccountInfo);

    const rootAccountRemote2 = await pahApi.apis.LocationToAccountApi.convert_location(XcmVersionedLocation.V5(TEER_FROM_COUSIN))
    if (!rootAccountRemote2.success) {
        console.error("Failed to get root account on remote2 chain: ", rootAccountRemote2);
        await itkClient.destroy();
        await kahClient.destroy();
        await pahClient.destroy();
        return;
    }
    const sovereignAccountInfoRemote2 = await itkApi.query.System.Account.getValue(rootAccountLocal.value)
    console.log("Sovereign account on remote2 chain: ", rootAccountRemote2, " with AccountInfo ", sovereignAccountInfoRemote2);

    const weightRes = await itkApi.apis.XcmPaymentApi.query_xcm_weight(tentativeXcm);
    if (!weightRes.success) {
        console.error("Failed to get weight for tentative XCM: ", weightRes);
        await itkClient.destroy();
        await kahClient.destroy();
        await pahClient.destroy();
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
    const [localFeesEstimate, remote1FeesEstimate, remote2FeesEstimateKsm] =
        (await estimateFees(tentativeXcm))!;

    console.log("Local fees estimate [TEER]: ", localFeesEstimate);
    console.log("Remote 1 fees estimate [TEER]: ", remote1FeesEstimate);
    console.log("Remote 2 fees estimate [KSM]: ", remote2FeesEstimateKsm);
    // With these estimates, we can create the final XCM to execute.
    const xcm = await createXcm(
        transferAmount,
        localFeesEstimate,
        remote1FeesEstimate,
        BigInt(Math.round(Number(remote2FeesEstimateKsm) * teerPerKsm * 1.5)),
        remote2FeesEstimateKsm
    );

    const weightResult = await itkApi.apis.XcmPaymentApi.query_xcm_weight(xcm);
    if (weightResult.success) {
        const tx = itkApi.tx.PolkadotXcm.execute({
            message: xcm,
            max_weight: weightResult.value,
        });
        const stx = await itkApi.tx.Sudo.sudo({call: tx.decodedCall})
        console.log("encoded adjusted call on source chain (e.g. to try with chopsticks): ", (await stx.getEncodedData()).asHex());
        const signer = getAliceSigner();
        console.log("Executing XCM now....")
        const result = await stx.signAndSubmit(signer);
        console.dir(stringify(result.txHash));
        console.log("Await System.Remarked event on destination chain...")
        await pahApi.event.System.Remarked.watch()
            .pipe(take(1))
            .forEach((event) => {
                console.log("Event received: System.Remarked from ", event.payload.sender, " with hash ", event.payload.hash.asHex());
            });
        const issuedEvents = await kahApi.event.ForeignAssets.Issued.pull();
        if (issuedEvents.length > 0) {
            console.log("foreignAssets.Issued (returning overpaid fees to sovereign sibling account on KAH [TEER]", issuedEvents[0].payload.amount, " to ", issuedEvents[0].payload.owner);
            const effectiveFees = remote1FeesEstimate - issuedEvents[0].payload.amount
            console.log("Effectively paid fees ", effectiveFees, " = ", Number(effectiveFees) / Number(TEER_UNITS), " TEER");
        } else {
            console.log("No foreignAssets.issued events found. this likely means it didn't work as expected.");
        }


    }
    await itkClient.destroy();
    await kahClient.destroy();
    await pahClient.destroy();
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
    remote1FeesTeer: bigint,
    remote2AllocationTeer: bigint,
    remote2FeesKsm: bigint,
): Promise<XcmVersionedXcm> {
    const executeOnPah = pahApi.tx.System.remark_with_event({remark: Binary.fromText("Hello Polkadot")})
    const teerToWithdraw = {
        id: TEER_FROM_SELF,
        fun: XcmV3MultiassetFungibility.Fungible(
            transferAmount + localFees + remote1FeesTeer + remote2AllocationTeer,
        ),
    };
    const teerForRemote1Total = {
        id: TEER_FROM_SELF,
        fun: XcmV3MultiassetFungibility.Fungible(remote1FeesTeer + remote2AllocationTeer),
    };
    const teerToSwapOnRemote1 = {
        id: TEER_FROM_SIBLING,
        fun: XcmV3MultiassetFungibility.Fungible(remote2AllocationTeer),
    };
    const ksmForRemote2Fees = {
        id: KSM_FROM_KUSAMA_PARACHAINS,
        fun: XcmV3MultiassetFungibility.Fungible(remote2FeesKsm),
    };
    const teerForRemote1Filter = XcmV5AssetFilter.Definite([teerForRemote1Total]);
    const teerToSwapOnRemote1Filter = XcmV5AssetFilter.Definite([teerToSwapOnRemote1]);
    const ksmForRemote2Filter = XcmV5AssetFilter.Definite([ksmForRemote2Fees]);

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
            remote_fees: Enum("Teleport", teerForRemote1Filter),
            assets: [],
            //assets: [Enum("Teleport", teerToSwapOnRemote1Filter)],
            remote_xcm: [
                XcmV5Instruction.SetAppendix([
                    XcmV5Instruction.RefundSurplus(),
                    XcmV5Instruction.DepositAsset({
                        assets: XcmV5AssetFilter.Wild(XcmV5WildAsset.All()),
                        beneficiary: TEER_FROM_SIBLING,
                    })
                ]),
                // we 'd like exactly ksmForRemote2Fees and keep the rest in TEER
                // XcmV5Instruction.ExchangeAsset({
                //     give: teerToSwapOnRemote1Filter,
                //     want: [ksmForRemote2Fees],
                //     maximal: false
                // }),
                // as a shortcut, let's use KSM we already have on sovereign account
                XcmV5Instruction.WithdrawAsset([ksmForRemote2Fees]),
                XcmV5Instruction.InitiateTransfer({
                    destination: PAH_FROM_KAH,
                    preserve_origin: true,
                    remote_fees: Enum("ReserveDeposit", ksmForRemote2Filter),
                    assets: [],
                    remote_xcm: [
                        XcmV5Instruction.SetAppendix([
                            XcmV5Instruction.RefundSurplus(),
                            XcmV5Instruction.DepositAsset({
                                assets: XcmV5AssetFilter.Wild(XcmV5WildAsset.All()),
                                beneficiary: TEER_FROM_COUSIN,
                            })
                        ]),
                        XcmV5Instruction.Transact({
                            origin_kind: XcmV2OriginKind.SovereignAccount(),
                            call: await executeOnPah.getEncodedData(),
                        }),
                    ],
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
): Promise<[bigint, bigint, bigint] | undefined> {
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
    const messageToKah = messages[0];

    const exchangeAssetInstruction = messageToKah?.value.find((instruction: any) =>
        instruction.type === "ExchangeAsset"
    )
    if (
        exchangeAssetInstruction &&
        typeof exchangeAssetInstruction.value === "object" &&
        exchangeAssetInstruction.value !== null &&
        "give" in exchangeAssetInstruction.value &&
        "want" in exchangeAssetInstruction.value
    ) {
        console.log("ExchangeAsset attempt: give ", exchangeAssetInstruction.value.give.value);
        console.log("ExchangeAsset attempt: want ", exchangeAssetInstruction.value.want[0]);
    } else {
        console.log("ExchangeAsset instruction does not have 'give' or 'want' properties:", exchangeAssetInstruction?.value);
    }

    // We're only dealing with version 4.
    if (messageToKah?.type !== "V5") {
        console.error("messageToKah failed: expected XCMv5");
        return;
    }

    // We get the delivery fees using the size of the forwarded xcm.
    const deliveryFees = await itkApi.apis.XcmPaymentApi.query_delivery_fees(
        XcmVersionedLocation.V5(KAH_FROM_IK),
        messageToKah,
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
    console.log("deliveryFees to remote1: ", deliveryFees.value.value);

    // Local fees are execution + delivery.
    const localFees = executionFees.value + deliveryFees.value.value[0].fun.value;

    // Now we dry run on the destination.
    const remote1DryRunResult = await kahApi.apis.DryRunApi.dry_run_xcm(
        XcmVersionedLocation.V5({
            parents: 1,
            interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(IK_PARA_ID)),
        }),
        messageToKah,
    );
    if (
        !remote1DryRunResult.success ||
        remote1DryRunResult.value.execution_result.type !== "Complete"
    ) {
        console.error("remote1DryRunResult failed: ", remote1DryRunResult);
        return;
    }
    console.log("remote1DryRunResult: ", remote1DryRunResult.value);
    const swapCreditEvent = remote1DryRunResult.value.emitted_events.find(
        (event: any) =>
            event.type === "AssetConversion" &&
            event.value?.type === "SwapCreditExecuted"
    );

    if (
        swapCreditEvent &&
        typeof swapCreditEvent.value.value === "object" &&
        swapCreditEvent.value.value !== null &&
        "amount_in" in swapCreditEvent.value.value &&
        "amount_out" in swapCreditEvent.value.value
    ) {
        console.log("Found SwapCreditExecuted event:", swapCreditEvent.value.value);
    } else {
        console.error("SwapCreditExecuted event not found or malformed.", swapCreditEvent);
        return;
    }
    const teerPerKsm = Number(swapCreditEvent.value?.value?.amount_in) / Number(swapCreditEvent?.value?.value?.amount_out);
    const teerSpent = swapCreditEvent.value?.value?.amount_in;

    const remote1Weight =
        await kahApi.apis.XcmPaymentApi.query_xcm_weight(messageToKah);
    if (!remote1Weight.success) {
        console.error("remote1Weight failed: ", remote1Weight);
        return;
    }
    console.log("remote1Weight: ", remote1Weight.value);

    // Remote fees are only execution.
    const remote1FeesInKsm =
        await kahApi.apis.XcmPaymentApi.query_weight_to_asset_fee(
            remote1Weight.value,
            XcmVersionedAssetId.V5(KSM_FROM_KUSAMA_PARACHAINS),
        );

    if (!remote1FeesInKsm.success) {
        console.error("remote1FeesInKsm failed: ", remote1FeesInKsm);
        return;
    }

    // XCM execution might result in multiple messages being sent.
    // That's why we need to search for our message in the `forwarded_xcms` array.
    const [_dummy, messages2] = remote1DryRunResult.value.forwarded_xcms.find(
        ([location, message]) =>
            location.type === "V5"
    )!;

    const messageToKbh = messages2[0];
    // Found it.
    console.log("messageToKbh: ", messageToKbh);

    const exportMessageInstruction = messageToKbh?.value.find((instruction: any) =>
        instruction.type === "ExportMessage" &&
        typeof instruction.value === "object" &&
        instruction.value !== null &&
        "xcm" in instruction.value
    )
    console.log(exportMessageInstruction)
    const messageToPah = {
        type: messageToKbh.type,
        value: exportMessageInstruction?.value?.xcm
    };
    console.log("messageToPah: ", messageToPah);

    // Now we dry run on the destination.
    const remote2DryRunResult = await pahApi.apis.DryRunApi.dry_run_xcm(
        // XCM origin has to be KAH. It will then AliasOrigin into IK upon execution
        // see runtime patch to allow this: https://github.com/polkadot-fellows/runtimes/compare/main...encointer:runtimes:ab/trusted-aliaser-patch
        XcmVersionedLocation.V5({
            parents: 2,
            interior: XcmV5Junctions.X2([
                XcmV5Junction.GlobalConsensus(XcmV5NetworkId.Kusama()),
                XcmV5Junction.Parachain(KAH_PARA_ID)
            ]),
        }),
        messageToPah,
    );
    if (
        !remote2DryRunResult.success ||
        remote2DryRunResult.value.execution_result.type !== "Complete"
    ) {
        console.error("remote2DryRunResult failed: ", remote2DryRunResult);
        return;
    }
    console.log("remote2DryRunResult: ", remote2DryRunResult.value);

    const swapCreditEvent2 = remote2DryRunResult.value.emitted_events.find(
        (event: any) =>
            event.type === "AssetConversion" &&
            event.value?.type === "SwapCreditExecuted"
    );

    if (
        swapCreditEvent2 &&
        typeof swapCreditEvent2.value.value === "object" &&
        swapCreditEvent2.value.value !== null &&
        "amount_in" in swapCreditEvent2.value.value &&
        "amount_out" in swapCreditEvent2.value.value
    ) {
        console.log("Found SwapCreditExecuted event:", swapCreditEvent2.value.value);
    } else {
        console.error("SwapCreditExecuted event not found or malformed.", swapCreditEvent2);
        return;
    }
    const ksmPerDot = Number(swapCreditEvent2.value?.value?.amount_in) / Number(swapCreditEvent2?.value?.value?.amount_out);
    const ksmSpent = swapCreditEvent2.value?.value?.amount_in;

    const remote2Weight =
        await pahApi.apis.XcmPaymentApi.query_xcm_weight(messageToKah);
    if (!remote2Weight.success) {
        console.error("remote2Weight failed: ", remote2Weight);
        return;
    }
    console.log("API: remote2Weight: ", remote2Weight.value);

    // Remote fees are only execution.
    const remote2FeesInDot =
        await pahApi.apis.XcmPaymentApi.query_weight_to_asset_fee(
            remote2Weight.value,
            XcmVersionedAssetId.V5(KSM_FROM_KUSAMA_PARACHAINS),
        );

    if (!remote2FeesInDot.success) {
        console.error("remote2FeesInDot failed: ", remote2FeesInDot);
        return;
    }
    console.log("API: remote1FeesInKsm: ", remote1FeesInKsm.value);
    console.log("API: remote2FeesInDot: ", remote2FeesInDot.value);

    console.log("simulated rate as TEER per KSM: ", teerPerKsm, " with TEER converted for fees: ", teerSpent, " equal to fees in KSM: ", swapCreditEvent.value.value.amount_out);
    console.log("simulated rate as KSM per DOT: ", ksmPerDot, " with KSM converted for fees: ", ksmSpent, " equal to fees in DOT: ", swapCreditEvent2.value.value.amount_out);


    const remote1FeesInTeer = BigInt(Math.round(Number(teerSpent) * 1.1));
    console.log("remote1FeesInTeer (with margin): ", remote1FeesInTeer);

    const remote2FeesInKsm = BigInt(Math.round(Number(ksmSpent) * 1.1));
    console.log("remote2FeesInKsm (with margin): ", remote2FeesInKsm);

    console.log("to be paid by caller to cover everything: ", localFees + remote1FeesInTeer + BigInt(Math.round(Number(remote2FeesInKsm) * teerPerKsm)), " TEER");

    return [localFees, remote1FeesInTeer, remote2FeesInKsm];
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