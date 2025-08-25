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
    itp, // bun papi add itp -w http://localhost:8003
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
const IP_PARA_ID = 2039;
const WATCHDOG_ACCOUNT = "2P2pRoXYwZAWVPXXtR6is5o7L34Me72iuNdiMZxeNV2BkgsH"; // Alice

// if false, we assume zombienet
const CHOPSTICKS: boolean = false;

const DIRECTION = "IK>IP";
// const DIRECTION = "IP>IK";

// We're running against chopsticks with wasm-override to get XCMv5 support.
// `npx @acala-network/chopsticks@latest xcm --p=kusama-asset-hub --p=./configs/integritee-kusama.yml`
const KAH_WS_URL = CHOPSTICKS
    ? "ws://localhost:8000"
    : "ws://localhost:9010";
const IK_WS_URL = CHOPSTICKS
    ? "ws://localhost:8001"
    : "ws://localhost:9144"
const PAH_WS_URL = CHOPSTICKS
    ? "ws://localhost:8002"
    : "ws://localhost:9910"
const IP_WS_URL = CHOPSTICKS
    ? "ws://localhost:8003"
    : "ws://localhost:9244"

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
const DOT_UNITS = 10_000_000_000n;

const KSM_FROM_KUSAMA_PARACHAINS = {
    parents: 1,
    interior: XcmV5Junctions.Here(),
};
const KSM_FROM_POLKADOT_PARACHAINS = {
    parents: 2,
    interior: XcmV5Junctions.X1(XcmV5Junction.GlobalConsensus(XcmV5NetworkId.Kusama())),
};
const DOT_FROM_POLKADOT_PARACHAINS = {
    parents: 1,
    interior: XcmV5Junctions.Here(),
};
const DOT_FROM_KUSAMA_PARACHAINS = {
    parents: 2,
    interior: XcmV5Junctions.X2([XcmV5Junction.GlobalConsensus(XcmV5NetworkId.Polkadot()), XcmV5Junction.Parachain(PAH_PARA_ID)]),
};
const TEER_FROM_SELF = {
    parents: 0,
    interior: XcmV5Junctions.Here(),
};
const ITK_FROM_SIBLING = {
    parents: 1,
    interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(2015)),
};
const ITK_FROM_COUSIN = {
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

const itpClient = createClient(
    withPolkadotSdkCompat(getWsProvider(IP_WS_URL)),
);
const itpApi = itpClient.getTypedApi(itp);

const portPlan = {
    source: {
        api: itkApi,
        name: "ITK",
        para_id: IK_PARA_ID,
        native_units: TEER_UNITS,
        native_symbol: "TEER",
        native_from_sibling: ITK_FROM_SIBLING,
        native_from_cousin: ITK_FROM_COUSIN,
        sovereign_self: TEER_FROM_SELF,
    },
    sourceAH: {
        api: kahApi,
        name: "KAH",
        para_id: KAH_PARA_ID,
        native_units: KSM_UNITS,
        native_symbol: "KSM",
        native_from_sibling: KSM_FROM_KUSAMA_PARACHAINS,
        native_from_cousin: KSM_FROM_POLKADOT_PARACHAINS
    },
    destinationAH: {
        api: pahApi,
        name: "PAH",
        para_id: PAH_PARA_ID,
        native_units: DOT_UNITS,
        native_symbol: "DOT",
        native_from_sibling: DOT_FROM_POLKADOT_PARACHAINS,
        native_from_cousin: DOT_FROM_KUSAMA_PARACHAINS
    },
    destination: {
        api: itpApi,
        name: "ITP",
        para_id: IP_PARA_ID,
        native_units: TEER_UNITS,
    },
    destroy: () => {
        return Promise.all([
            itkClient.destroy(),
            kahClient.destroy(),
            pahClient.destroy(),
            itpClient.destroy(),
        ]);
    }
}

// The whole execution of the script.
main(portPlan);

// We'll teleport KSM from Asset Hub to People.
// Using the XcmPaymentApi and DryRunApi, we'll estimate the XCM fees accurately.
async function main(plan: any) {
    // The amount of TEER we wish to teleport besides paying fees.
    const transferAmount = 1000000000000n;

    if (CHOPSTICKS) {
        const stx = await plan.source.api.tx.System.remark_with_event({remark: Binary.fromText("Let's trigger state migration")})
        const signer = getAliceSigner();
        await stx.signAndSubmit(signer);
        const stx2 = await plan.destination.api.tx.System.remark_with_event({remark: Binary.fromText("Let's trigger state migration")})
        await stx2.signAndSubmit(signer);
        console.log("triggered state migrations if necessary. waiting for a bit....")
        // wait for chopsticks api's to catch up
        await new Promise(resolve => setTimeout(resolve, 5000));
    }

    const referenceAmountTeer = 1000000000000n;
    const remote2FeesHighEstimateTeerConverted = await plan.sourceAH.api.apis.AssetConversionApi.quote_price_tokens_for_exact_tokens(plan.source.native_from_sibling, plan.sourceAH.native_from_sibling, referenceAmountTeer, true);
    const teerPerKsm = Number(remote2FeesHighEstimateTeerConverted) / Number(referenceAmountTeer)
    console.log(`Current AssetConversion quote on ${plan.sourceAH.name}: out: `, remote2FeesHighEstimateTeerConverted, " in ", referenceAmountTeer, ` ${plan.source.native_symbol}. price: `, teerPerKsm, ` ${plan.source.native_symbol} per ${plan.sourceAH.native_symbol}`);

    const referenceAmountKsm = 100000000000n;
    const remote3FeesHighEstimateKsmConverted = await pahApi.apis.AssetConversionApi.quote_price_tokens_for_exact_tokens(plan.sourceAH.native_from_cousin, plan.destinationAH.native_from_sibling, referenceAmountKsm, true);
    const ksmPerDot = Number(remote3FeesHighEstimateKsmConverted) / Number(referenceAmountKsm)
    console.log(`Current AssetConversion quote for ${plan.destinationAH.name} account: out: `, remote3FeesHighEstimateKsmConverted, " in ", referenceAmountKsm, ` ${plan.sourceAH.native_symbol}. price: `, ksmPerDot / 100.0, ` ${plan.sourceAH.native_symbol} per ${plan.destinationAH.native_symbol}`);

    // We can now query the root account on the source chain.
    const rootAccountLocal = await plan.source.api.apis.LocationToAccountApi.convert_location(XcmVersionedLocation.V5(plan.source.sovereign_self))
    if (!rootAccountLocal.success) {
        console.error("Failed to get root account on source chain: ", rootAccountLocal);
        await plan.destroy();
        return;
    }
    const rootAccountInfo = await plan.source.api.query.System.Account.getValue(rootAccountLocal.value)
    console.log("Root account on source chain: ", rootAccountLocal, " with AccountInfo ", rootAccountInfo);

    const rootAccountRemote2 = await plan.destinationAH.api.apis.LocationToAccountApi.convert_location(XcmVersionedLocation.V5(plan.source.native_from_cousin))
    if (!rootAccountRemote2.success) {
        console.error("Failed to get root account on remote2 chain: ", rootAccountRemote2);
        await plan.destroy();
        return;
    }
    const sovereignAccountInfoRemote2 = await plan.source.api.query.System.Account.getValue(rootAccountLocal.value)
    console.log(`Sovereign account of ITK on remote2 ITP chain: `, rootAccountRemote2, " with AccountInfo ", sovereignAccountInfoRemote2);

    // the actual extrinsic we would send to bridge TEER from IK to IP
    const portTokensTx = plan.source.api.tx.Porteer.port_tokens({
        amount: transferAmount,
        forwardTokensToLocation: null
    });
    // const setFeesTx = plan.source.api.tx.Porteer.set_xcm_fee_params({
    //     fees: {
    //         hop1: 701987734047n,
    //         hop2: 97085698579n,
    //         hop3: 4886724760n
    //     }
    // });
    const watchdogTx = plan.source.api.tx.Porteer.watchdog_heartbeat([]);
    const calls = [watchdogTx.decodedCall, portTokensTx.decodedCall];
    const batchTx = plan.source.api.tx.Utility.batch({calls: calls});
    // console.log("tentative call on source chain (e.g. to try with chopsticks): ", batchTx.decodedCall);

    console.log("encoded tentative call on source chain (e.g. to try with chopsticks): ", (await batchTx.getEncodedData()).asHex());

    // This will give us the adjusted estimates, much more accurate than before.
    const [localFeesEstimate, remote1FeesEstimate, remote2FeesEstimateKsm, remote3FeesEstimateDot] =
        (await estimateFees(plan, batchTx))!;

    console.log(`Local fees estimate [TEER]: `, localFeesEstimate);
    console.log(`Remote 1 fees estimate [TEER]: `, remote1FeesEstimate);
    console.log(`Remote 2 fees estimate [KSM]: `, remote2FeesEstimateKsm);
    console.log(`Remote 3 fees estimate [DOT]: `, remote3FeesEstimateDot);

    const setFeesTx = plan.source.api.tx.Porteer.set_xcm_fee_params({
        fees: {
            hop1: 701987734047n,
            hop2: 97085698579n,
            hop3: 4886724760n
        }
    });
    const setFeesProposalTx = plan.source.api.tx.TechnicalCommittee.propose({
        threshold: 2,
        proposal: setFeesTx.decodedCall,
        length_bound: 51,
    });
    console.log("encoded suggested call to set updated fees on source chain (to propose to TC / PorteerAdmin): ", (await setFeesTx.getEncodedData()).asHex());
    console.log("encoded proposal to TC (to submit as member of TC): ", (await setFeesProposalTx.getEncodedData()).asHex());

    await plan.destroy();
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

// Estimates both local and remote fees for a given XCM-triggering extrinsic on source.
//
// This is the mechanism showcased on this script.
// Uses the XcmPaymentApi to get local fees, both execution and delivery.
// Then uses the DryRunApi to get the sent XCM and estimates remote fees
// connecting to the destination chain.
//
// If there's any issue and fees couldn't be estimated, returns undefined.
async function estimateFees(
    plan: any,
    tx: any,
): Promise<[bigint, bigint, bigint, bigint] | undefined> {

    const dryRunResult = await plan.source.api.apis.DryRunApi.dry_run_call(
        Enum("system", DispatchRawOrigin.Signed(WATCHDOG_ACCOUNT)),
        tx.decodedCall,
        XCM_VERSION,
    );
    if (!dryRunResult.success || !dryRunResult.value.execution_result.success) {
        console.error("dryRunResult failed: ", dryRunResult);
        console.error("localDryRun Error: ", dryRunResult.value.execution_result);
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
            location.value.interior.value.value === plan.sourceAH.para_id,
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
    const deliveryFees = await plan.source.api.apis.XcmPaymentApi.query_delivery_fees(
        XcmVersionedLocation.V5({
            parents: 1,
            interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(plan.sourceAH.para_id)),
        }),
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
    const deliveryFeesToRemote1Teer = deliveryFees.value.value[0].fun.value;
    console.log("deliveryFees to remote1 [TEER]: ", deliveryFeesToRemote1Teer);

    // Local fees for execution (which is virtual as root won't pay execution).
    const localExecutionFees = 0n;

    // Now we dry run on the destination.
    const remote1DryRunResult = await plan.sourceAH.api.apis.DryRunApi.dry_run_xcm(
        XcmVersionedLocation.V5({
            parents: 1,
            interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(plan.source.para_id)),
        }),
        messageToKah,
    );
    if (
        !remote1DryRunResult.success ||
        remote1DryRunResult.value.execution_result.type !== "Complete"
    ) {
        console.error("remote1DryRunResult failed: ", remote1DryRunResult);
        console.error("remote1DryRun Error: ", remote1DryRunResult.value.execution_result);
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
        await plan.sourceAH.api.apis.XcmPaymentApi.query_xcm_weight(messageToKah);
    if (!remote1Weight.success) {
        console.error("remote1Weight failed: ", remote1Weight);
        return;
    }
    console.log("remote1Weight: ", remote1Weight.value);

    // Remote fees are only execution.
    const remote1FeesInKsm =
        await plan.sourceAH.api.apis.XcmPaymentApi.query_weight_to_asset_fee(
            remote1Weight.value,
            XcmVersionedAssetId.V5(plan.sourceAH.native_from_sibling),
        );

    if (!remote1FeesInKsm.success) {
        console.error(`remote1FeesInKsm failed: `, remote1FeesInKsm);
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

    // We get the delivery fees using the size of the forwarded xcm.
    const deliveryFeesToRemote2 = await plan.sourceAH.api.apis.XcmPaymentApi.query_delivery_fees(
        XcmVersionedLocation.V5(plan.destinationAH.native_from_cousin),
        messageToKbh, // unclear if this should be messagetoKbh or Pah. doesn't seem to include bridge relayer fee
    );
    // Fees should be of the version we expect and fungible tokens, in particular, KSM.
    if (
        !deliveryFeesToRemote2.success ||
        deliveryFeesToRemote2.value.type !== "V5" ||
        deliveryFeesToRemote2.value.value.length < 1 ||
        deliveryFeesToRemote2.value.value[0]?.fun?.type !== "Fungible"
    ) {
        console.error("deliveryFeesToRemote2 failed: ", deliveryFeesToRemote2);
        return;
    }
    const deliveryFeesToRemote2Ksm = deliveryFeesToRemote2.value.value[0].fun.value
    console.log(`deliveryFees KAH to PAH [KSM]: `, deliveryFeesToRemote2Ksm);

    // Now we dry run on the destination.
    const remote2DryRunResult = await plan.destinationAH.api.apis.DryRunApi.dry_run_xcm(
        // XCM origin has to be KAH. It will then AliasOrigin into IK upon execution
        // see runtime patch to allow this: https://github.com/polkadot-fellows/runtimes/compare/main...encointer:runtimes:ab/trusted-aliaser-patch
        XcmVersionedLocation.V5({
            parents: 2,
            interior: XcmV5Junctions.X2([
                XcmV5Junction.GlobalConsensus(XcmV5NetworkId.Kusama()),
                XcmV5Junction.Parachain(plan.sourceAH.para_id)
            ]),
        }),
        messageToPah,
    );
    if (
        !remote2DryRunResult.success ||
        remote2DryRunResult.value.execution_result.type !== "Complete"
    ) {
        console.error("remote2DryRunResult failed: ", remote2DryRunResult);
        console.error("remote2DryRun Error: ", remote2DryRunResult.value.execution_result);
        return;
    }
    console.log("remote2DryRunResult: ", remote2DryRunResult.value);

    // XCM execution might result in multiple messages being sent.
    // That's why we need to search for our message in the `forwarded_xcms` array.
    const [_dummy3, messages3] = remote2DryRunResult.value.forwarded_xcms.find(
        ([location, _]) =>
            location.type === "V5" &&
            location.value.parents === 1 &&
            location.value.interior.type === "X1" &&
            location.value.interior.value.type === "Parachain" &&
            location.value.interior.value.value === plan.destination.para_id,
    )!;
    // Found it.
    const messageToItp = messages3[0];
    console.log("messageToItp: ", messageToItp);

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
        await plan.destinationAH.api.apis.XcmPaymentApi.query_xcm_weight(messageToKah);
    if (!remote2Weight.success) {
        console.error("remote2Weight failed: ", remote2Weight);
        return;
    }
    console.log("API: remote2Weight: ", remote2Weight.value);

    // Remote fees are only execution.
    const remote2FeesInDot =
        await plan.destinationAH.api.apis.XcmPaymentApi.query_weight_to_asset_fee(
            remote2Weight.value,
            XcmVersionedAssetId.V5(plan.destinationAH.native_from_sibling),
        );

    if (!remote2FeesInDot.success) {
        console.error(`remote2FeesInDot failed: `, remote2FeesInDot);
        return;
    }

    // Now we dry run on the destination.
    const remote3DryRunResult = await plan.destination.api.apis.DryRunApi.dry_run_xcm(
        // XCM origin has to be KAH. It will then AliasOrigin into IK upon execution
        // see runtime patch to allow this: https://github.com/polkadot-fellows/runtimes/compare/main...encointer:runtimes:ab/trusted-aliaser-patch
        XcmVersionedLocation.V5({
            parents: 1,
            interior: XcmV5Junctions.X1(
                XcmV5Junction.Parachain(plan.destinationAH.para_id)
            ),
        }),
        messageToItp,
    );
    if (
        !remote3DryRunResult.success ||
        remote3DryRunResult.value.execution_result.type !== "Complete"
    ) {
        console.error("remote3DryRunResult failed: ", remote3DryRunResult);
        console.error("remote3DryRun Error: ", remote3DryRunResult.value.execution_result);
        return;
    }
    console.log("remote3DryRunResult: ", remote3DryRunResult.value);

    // We get the delivery fees using the size of the forwarded xcm.
    const deliveryFeesToRemote3 = await plan.destinationAH.api.apis.XcmPaymentApi.query_delivery_fees(
        XcmVersionedLocation.V5({
            parents: 1,
            interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(plan.destination.para_id)),
        }),
        messageToItp,
    );
    // Fees should be of the version we expect and fungible tokens, in particular, KSM.
    if (
        !deliveryFeesToRemote3.success ||
        deliveryFeesToRemote3.value.type !== "V5" ||
        deliveryFeesToRemote3.value.value.length < 1 ||
        deliveryFeesToRemote3.value.value[0]?.fun?.type !== "Fungible"
    ) {
        console.error("deliveryFeesToRemote3 failed: ", deliveryFeesToRemote3);
        return;
    }
    const deliveryFeesToRemote3Dot = deliveryFeesToRemote3.value.value[0].fun.value
    console.log(`deliveryFees PAH to ITP [DOT]: `, deliveryFeesToRemote3Dot);

    const remote3Weight =
        await plan.destination.api.apis.XcmPaymentApi.query_xcm_weight(messageToItp);
    if (!remote3Weight.success) {
        console.error("remote3Weight failed: ", remote3Weight);
        return;
    }
    console.log("API: remote3Weight: ", remote3Weight.value);

    // Remote fees are only execution.
    const resultRemote3FeesInDot =
        await plan.destination.api.apis.XcmPaymentApi.query_weight_to_asset_fee(
            remote3Weight.value,
            XcmVersionedAssetId.V5(plan.destinationAH.native_from_sibling),
        );

    if (!resultRemote3FeesInDot.success) {
        console.error(`remote3FeesInDot failed: `, resultRemote3FeesInDot);
        return;
    }
    const remote3FeesInDot = 4580824760n //TODO: weight_2_fee on ITP seems off:  resultRemote3FeesInDot.value. See: https://github.com/integritee-network/parachain/issues/329

    console.log(`API: localExecutionFees (virtual) [TEER]: `, localExecutionFees);
    console.log(`API: deliveryFeesToRemote1Teer    [TEER]: `, deliveryFeesToRemote1Teer);
    console.log(`API: remote1FeesInKsm*            [KSM] : `, remote1FeesInKsm.value);
    console.log(`API: deliveryFeesToRemote2Ksm     [KSM] : `, deliveryFeesToRemote2Ksm);
    console.log(`API: remote2FeesInDot*            [DOT] : `, remote2FeesInDot.value);
    console.log(`API: deliveryFeesToRemote3Dot     [DOT] : `, deliveryFeesToRemote3Dot);
    console.log(`API: remote3FeesInDot             [DOT] : `, remote3FeesInDot);

    console.log(`simulated rate as TEER per KSM: `, teerPerKsm, ` with TEER converted for fees: `, teerSpent, ` equal to fees in KSM: `, swapCreditEvent.value.value.amount_out);
    console.log(`simulated rate as KSM per DOT: `, ksmPerDot / 100, ` with KSM converted for fees: `, ksmSpent, ` equal to fees in DOT: `, swapCreditEvent2.value.value.amount_out);


    const remote1FeesInTeer = BigInt(Math.round(Number(remote1FeesInKsm.value + deliveryFeesToRemote2Ksm) * teerPerKsm * 1.1));
    console.log(`remote1FeesInTeer (with margin*): `, remote1FeesInTeer);

    const remote2FeesInKsm = BigInt(Math.round(Number(remote2FeesInDot.value + deliveryFeesToRemote3Dot) * ksmPerDot * 1.1));
    console.log(`remote2FeesInKsm (with margin*): `, remote2FeesInKsm);

    const totalCallerFeesInTeer = localExecutionFees +
        deliveryFeesToRemote1Teer + remote1FeesInTeer +
        BigInt(Math.round(Number(remote2FeesInKsm + deliveryFeesToRemote2Ksm) * teerPerKsm +
            Number(remote3FeesInDot + deliveryFeesToRemote3Dot) * teerPerKsm * ksmPerDot))
    console.log("to be paid by caller to cover everything: ", Number(totalCallerFeesInTeer) / Number(TEER_UNITS), ` TEER`);

    return [deliveryFeesToRemote1Teer, remote1FeesInTeer, remote2FeesInKsm, remote3FeesInDot + deliveryFeesToRemote3Dot];
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