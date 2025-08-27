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
    itk, // bun papi add itk -w http://localhost:8001 | bun papi add itk -w http://localhost:9144
    itp, // bun papi add itp -w http://localhost:8003 | bun papi add itp -w http://localhost:9244
    kah, // bun papi add kah -w http://localhost:8000 | bun papi add kah -w http://localhost:9010
    pah, // bun papi add pah -w http://localhost:8002 | bun papi add pah -w http://localhost:9910
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

//const DIRECTION = "IK>IP";
const DIRECTION = "IP>IK";

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
const KAH_FROM_KUSAMA_PARACHAINS = {
    parents: 1,
    interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(KAH_PARA_ID)),
};
const KAH_FROM_POLKADOT_PARACHAINS = {
    parents: 2,
    interior: XcmV5Junctions.X2([XcmV5Junction.GlobalConsensus(XcmV5NetworkId.Kusama()), XcmV5Junction.Parachain(KAH_PARA_ID)]),
};
const DOT_FROM_POLKADOT_PARACHAINS = {
    parents: 1,
    interior: XcmV5Junctions.Here(),
};
const DOT_FROM_KUSAMA_PARACHAINS = {
    parents: 2,
    interior: XcmV5Junctions.X1(XcmV5Junction.GlobalConsensus(XcmV5NetworkId.Polkadot())),
};
const PAH_FROM_POLKADOT_PARACHAINS = {
    parents: 1,
    interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(PAH_PARA_ID)),
};
const PAH_FROM_KUSAMA_PARACHAINS = {
    parents: 2,
    interior: XcmV5Junctions.X2([XcmV5Junction.GlobalConsensus(XcmV5NetworkId.Polkadot()), XcmV5Junction.Parachain(PAH_PARA_ID)]),
};
const TEER_FROM_SELF = {
    parents: 0,
    interior: XcmV5Junctions.Here(),
};
const ITK_FROM_SIBLING = {
    parents: 1,
    interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(IK_PARA_ID)),
};
const ITK_FROM_COUSIN = {
    parents: 2,
    interior: XcmV5Junctions.X2([XcmV5Junction.GlobalConsensus(XcmV5NetworkId.Kusama()), XcmV5Junction.Parachain(IK_PARA_ID)]),
};
const ITP_FROM_SIBLING = {
    parents: 1,
    interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(IP_PARA_ID)),
};
const ITP_FROM_COUSIN = {
    parents: 2,
    interior: XcmV5Junctions.X2([XcmV5Junction.GlobalConsensus(XcmV5NetworkId.Polkadot()), XcmV5Junction.Parachain(IP_PARA_ID)]),
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

const portPlanK2P = {
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
        native_from_cousin: KSM_FROM_POLKADOT_PARACHAINS,
        self_from_cousin: KAH_FROM_POLKADOT_PARACHAINS
    },
    destinationAH: {
        api: pahApi,
        name: "PAH",
        para_id: PAH_PARA_ID,
        native_units: DOT_UNITS,
        native_symbol: "DOT",
        native_from_sibling: DOT_FROM_POLKADOT_PARACHAINS,
        native_from_cousin: DOT_FROM_KUSAMA_PARACHAINS,
        self_from_sibling: PAH_FROM_POLKADOT_PARACHAINS,
        self_from_cousin: PAH_FROM_KUSAMA_PARACHAINS,
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

const portPlanP2K = {
    source: {
        api: itpApi,
        name: "ITP",
        para_id: IP_PARA_ID,
        native_units: TEER_UNITS,
        native_symbol: "TEER",
        native_from_sibling: ITP_FROM_SIBLING,
        native_from_cousin: ITP_FROM_COUSIN,
        sovereign_self: TEER_FROM_SELF,
    },
    sourceAH: {
        api: pahApi,
        name: "PAH",
        para_id: PAH_PARA_ID,
        native_units: DOT_UNITS,
        native_symbol: "DOT",
        native_from_sibling: DOT_FROM_POLKADOT_PARACHAINS,
        native_from_cousin: DOT_FROM_KUSAMA_PARACHAINS,
        self_from_cousin: PAH_FROM_KUSAMA_PARACHAINS
    },
    destinationAH: {
        api: kahApi,
        name: "KAH",
        para_id: KAH_PARA_ID,
        native_units: KSM_UNITS,
        native_symbol: "KSM",
        native_from_sibling: KSM_FROM_KUSAMA_PARACHAINS,
        native_from_cousin: KSM_FROM_POLKADOT_PARACHAINS,
        self_from_sibling: KAH_FROM_KUSAMA_PARACHAINS,
        self_from_cousin: KAH_FROM_POLKADOT_PARACHAINS
    },
    destination: {
        api: itkApi,
        name: "ITK",
        para_id: IK_PARA_ID,
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
if (DIRECTION === "IK>IP") {
    main(portPlanK2P);
} else {
    main(portPlanP2K);
}


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
    const destinationAHFeesHighEstimateTeerConverted = await plan.sourceAH.api.apis.AssetConversionApi.quote_price_tokens_for_exact_tokens(plan.source.native_from_sibling, plan.sourceAH.native_from_sibling, referenceAmountTeer, true);
    const teerPerSourceAHNative = Number(destinationAHFeesHighEstimateTeerConverted) / Number(referenceAmountTeer)
    console.log(`Current AssetConversion quote on ${plan.sourceAH.name}: out: `, destinationAHFeesHighEstimateTeerConverted, " in ", referenceAmountTeer, ` ${plan.source.native_symbol}. price: `, teerPerSourceAHNative, ` ${plan.source.native_symbol} per ${plan.sourceAH.native_symbol}`);

    const referenceAmountSourceAHNative = 100000000000n;
    const destinationFeesHighEstimateSourceAHNativeConverted = await pahApi.apis.AssetConversionApi.quote_price_tokens_for_exact_tokens(plan.sourceAH.native_from_cousin, plan.destinationAH.native_from_sibling, referenceAmountSourceAHNative, true);
    const sourceAHNativePerDestinationAHNative = Number(destinationFeesHighEstimateSourceAHNativeConverted) / Number(referenceAmountSourceAHNative)
    console.log(`Current AssetConversion quote for ${plan.destinationAH.name} account: out: `, destinationFeesHighEstimateSourceAHNativeConverted, " in ", referenceAmountSourceAHNative, ` ${plan.sourceAH.native_symbol}. price: `, sourceAHNativePerDestinationAHNative / 100.0, ` ${plan.sourceAH.native_symbol} per ${plan.destinationAH.native_symbol}`);

    // We can now query the root account on the source chain.
    const rootAccountLocal = await plan.source.api.apis.LocationToAccountApi.convert_location(XcmVersionedLocation.V5(plan.source.sovereign_self))
    if (!rootAccountLocal.success) {
        console.error("Failed to get root account on source chain: ", rootAccountLocal);
        await plan.destroy();
        return;
    }
    const rootAccountInfo = await plan.source.api.query.System.Account.getValue(rootAccountLocal.value)
    console.log("Root account on source chain: ", rootAccountLocal, " with AccountInfo ", rootAccountInfo);

    const rootAccountDestinationAH = await plan.destinationAH.api.apis.LocationToAccountApi.convert_location(XcmVersionedLocation.V5(plan.source.native_from_cousin))
    if (!rootAccountDestinationAH.success) {
        console.error("Failed to get root account on destinationAH chain: ", rootAccountDestinationAH);
        await plan.destroy();
        return;
    }
    const sovereignAccountInfoDestinationAH = await plan.source.api.query.System.Account.getValue(rootAccountLocal.value)
    console.log(`Sovereign account of ${plan.source.name} on destinationAH ${plan.destination.name} chain: `, rootAccountDestinationAH, " with AccountInfo ", sovereignAccountInfoDestinationAH);

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
    const [localEquivalentFeesEstimate, sourceAHFeesEstimate, destinationAHFeesEstimateSourceAHNative, destinationFeesEstimateDestinationAHNative] =
        (await estimateFees(plan, batchTx))!;

    console.log(`Local equivalent fees estimate    [TEER]: `, localEquivalentFeesEstimate);
    console.log(`Remote 1 fees estimate [TEER]: `, sourceAHFeesEstimate);
    console.log(`Remote 2 fees estimate  [${plan.sourceAH.native_symbol}]: `, destinationAHFeesEstimateSourceAHNative);
    console.log(`Remote 3 fees estimate  [${plan.destinationAH.native_symbol}]: `, destinationFeesEstimateDestinationAHNative);

    const setFeesTx = plan.source.api.tx.Porteer.set_xcm_fee_params({
        fees: {
            local_equivalent_sum: localEquivalentFeesEstimate,
            hop1: sourceAHFeesEstimate,
            hop2: destinationAHFeesEstimateSourceAHNative,
            hop3: destinationFeesEstimateDestinationAHNative
        }
    });
    // const members = plan.source.api.query.TechnicalCommittee.Members.getValue();
    // console.log(members)
    const setFeesProposalTx = plan.source.api.tx.TechnicalCommittee.propose({
        threshold: 1,
        proposal: setFeesTx.decodedCall,
        length_bound: 51,
    });
    console.log(`${plan.source.name} encoded suggested call to set updated fees on source chain (to propose to TC / PorteerAdmin): `, (await setFeesTx.getEncodedData()).asHex());
    console.log(`${plan.source.name} encoded proposal to TC (to submit as member of TC): `, (await setFeesProposalTx.getEncodedData()).asHex());

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
    const messageToSourceAH = messages[0];

    const exchangeAssetInstruction = messageToSourceAH?.value.find((instruction: any) =>
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
    if (messageToSourceAH?.type !== "V5") {
        console.error("messageToSourceAH failed: expected XCMv5");
        return;
    }

    // We get the delivery fees using the size of the forwarded xcm.
    const deliveryFees = await plan.source.api.apis.XcmPaymentApi.query_delivery_fees(
        XcmVersionedLocation.V5({
            parents: 1,
            interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(plan.sourceAH.para_id)),
        }),
        messageToSourceAH,
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
    const deliveryFeesToSourceAHInTeer = deliveryFees.value.value[0].fun.value;
    console.log("deliveryFees to sourceAH [TEER]: ", deliveryFeesToSourceAHInTeer);

    // Local fees for execution (which is virtual as root won't pay execution).
    const localExecutionFees = 0n;

    // Now we dry run on the destination.
    const sourceAHDryRunResult = await plan.sourceAH.api.apis.DryRunApi.dry_run_xcm(
        XcmVersionedLocation.V5(plan.source.native_from_sibling),
        messageToSourceAH,
    );
    if (
        !sourceAHDryRunResult.success ||
        sourceAHDryRunResult.value.execution_result.type !== "Complete"
    ) {
        console.error("sourceAHDryRunResult failed: ", sourceAHDryRunResult);
        console.error("sourceAHDryRun Error: ", sourceAHDryRunResult.value.execution_result);
        return;
    }
    console.log("sourceAHDryRunResult: ", sourceAHDryRunResult.value);
    const swapCreditEvent = sourceAHDryRunResult.value.emitted_events.find(
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
    const teerPerSourceAHNative = Number(swapCreditEvent.value?.value?.amount_in) / Number(swapCreditEvent?.value?.value?.amount_out);
    const teerSpent = swapCreditEvent.value?.value?.amount_in;

    const sourceAHWeight =
        await plan.sourceAH.api.apis.XcmPaymentApi.query_xcm_weight(messageToSourceAH);
    if (!sourceAHWeight.success) {
        console.error("sourceAHWeight failed: ", sourceAHWeight);
        return;
    }
    console.log("sourceAHWeight: ", sourceAHWeight.value);

    // Remote fees are only execution.
    const sourceAHFeesInNative =
        await plan.sourceAH.api.apis.XcmPaymentApi.query_weight_to_asset_fee(
            sourceAHWeight.value,
            XcmVersionedAssetId.V5(plan.sourceAH.native_from_sibling),
        );

    if (!sourceAHFeesInNative.success) {
        console.error(`sourceAHFeesInNative failed: `, sourceAHFeesInNative);
        return;
    }

    // XCM execution might result in multiple messages being sent.
    // That's why we need to search for our message in the `forwarded_xcms` array.
    const [_dummy, messages2] = sourceAHDryRunResult.value.forwarded_xcms.find(
        ([location, message]) =>
            location.type === "V5"
    )!;

    const messageToSourceBH = messages2[0];
    // Found it.
    console.log("messageToSourceBH: ", messageToSourceBH);

    const exportMessageInstruction = messageToSourceBH?.value.find((instruction: any) =>
        instruction.type === "ExportMessage" &&
        typeof instruction.value === "object" &&
        instruction.value !== null &&
        "xcm" in instruction.value
    )
    console.log(exportMessageInstruction)
    const messageToDestinationAH = {
        type: messageToSourceBH.type,
        value: exportMessageInstruction?.value?.xcm
    };
    console.log("messageToDestinationAH: ", messageToDestinationAH);

    // We get the delivery fees using the size of the forwarded xcm.
    const deliveryFeesToDestinationAH = await plan.sourceAH.api.apis.XcmPaymentApi.query_delivery_fees(
        XcmVersionedLocation.V5(plan.destinationAH.self_from_cousin),
        messageToSourceBH, // unclear if this should be messageToSourceBH or destinationAH. doesn't seem to include bridge relayer fee
    );
    // Fees should be of the version we expect and fungible tokens, in particular, sourceRelayNative.
    if (
        !deliveryFeesToDestinationAH.success ||
        deliveryFeesToDestinationAH.value.type !== "V5" ||
        deliveryFeesToDestinationAH.value.value.length < 1 ||
        deliveryFeesToDestinationAH.value.value[0]?.fun?.type !== "Fungible"
    ) {
        console.error("deliveryFeesToDestinationAH failed: ", deliveryFeesToDestinationAH);
        return;
    }
    const deliveryFeesToDestinationAHInSourceAHNative = deliveryFeesToDestinationAH.value.value[0].fun.value
    console.log(`deliveryFees ${plan.sourceAH.name} to ${plan.destinationAH.name} [${plan.sourceAH.native_symbol}]: `, deliveryFeesToDestinationAHInSourceAHNative);

    // Now we dry run on the destination.
    const destinationAHDryRunResult = await plan.destinationAH.api.apis.DryRunApi.dry_run_xcm(
        // XCM origin has to be KAH. It will then AliasOrigin into IK upon execution
        // see runtime patch to allow this: https://github.com/polkadot-fellows/runtimes/compare/main...encointer:runtimes:ab/trusted-aliaser-patch
        XcmVersionedLocation.V5(plan.sourceAH.self_from_cousin),
        messageToDestinationAH,
    );
    if (
        !destinationAHDryRunResult.success ||
        destinationAHDryRunResult.value.execution_result.type !== "Complete"
    ) {
        console.error("destinationAHDryRunResult failed: ", destinationAHDryRunResult);
        console.error("destinationAHDryRun Error: ", destinationAHDryRunResult.value.execution_result);
        return;
    }
    console.log("destinationAHDryRunResult: ", destinationAHDryRunResult.value);

    // XCM execution might result in multiple messages being sent.
    // That's why we need to search for our message in the `forwarded_xcms` array.
    const [_dummy3, messages3] = destinationAHDryRunResult.value.forwarded_xcms.find(
        ([location, _]) =>
            location.type === "V5" &&
            location.value.parents === 1 &&
            location.value.interior.type === "X1" &&
            location.value.interior.value.type === "Parachain" &&
            location.value.interior.value.value === plan.destination.para_id,
    )!;
    // Found it.
    const messageToDestination = messages3[0];
    console.log("messageToDestination: ", messageToDestination);

    const swapCreditEvent2 = destinationAHDryRunResult.value.emitted_events.find(
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
    const sourceAHNativePerDestinationAHNative = Number(swapCreditEvent2.value?.value?.amount_in) / Number(swapCreditEvent2?.value?.value?.amount_out);
    const sourceAHNativeSpent = swapCreditEvent2.value?.value?.amount_in;

    const destinationAHWeight =
        await plan.destinationAH.api.apis.XcmPaymentApi.query_xcm_weight(messageToSourceAH);
    if (!destinationAHWeight.success) {
        console.error("destinationAHWeight failed: ", destinationAHWeight);
        return;
    }
    console.log("API: destinationAHWeight: ", destinationAHWeight.value);

    // Remote fees are only execution.
    const destinationAHFeesInNative =
        await plan.destinationAH.api.apis.XcmPaymentApi.query_weight_to_asset_fee(
            destinationAHWeight.value,
            XcmVersionedAssetId.V5(plan.destinationAH.native_from_sibling),
        );

    if (!destinationAHFeesInNative.success) {
        console.error(`destinationAHFeesInNative failed: `, destinationAHFeesInNative);
        return;
    }

    // Now we dry run on the destination.
    const destinationDryRunResult = await plan.destination.api.apis.DryRunApi.dry_run_xcm(
        // XCM origin has to be AH. It will then AliasOrigin into IK upon execution
        // see runtime patch to allow this: https://github.com/polkadot-fellows/runtimes/compare/main...encointer:runtimes:ab/trusted-aliaser-patch
        XcmVersionedLocation.V5(plan.destinationAH.self_from_sibling),
        messageToDestination,
    );
    if (
        !destinationDryRunResult.success ||
        destinationDryRunResult.value.execution_result.type !== "Complete"
    ) {
        console.error("destinationDryRunResult failed: ", destinationDryRunResult);
        console.error("destinationDryRun Error: ", destinationDryRunResult.value.execution_result);
        return;
    }
    console.log("destinationDryRunResult: ", destinationDryRunResult.value);

    // We get the delivery fees using the size of the forwarded xcm.
    const deliveryFeesToDestination = await plan.destinationAH.api.apis.XcmPaymentApi.query_delivery_fees(
        XcmVersionedLocation.V5({
            parents: 1,
            interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(plan.destination.para_id)),
        }),
        messageToDestination,
    );
    // Fees should be of the version we expect and fungible tokens, in particular, KSM.
    if (
        !deliveryFeesToDestination.success ||
        deliveryFeesToDestination.value.type !== "V5" ||
        deliveryFeesToDestination.value.value.length < 1 ||
        deliveryFeesToDestination.value.value[0]?.fun?.type !== "Fungible"
    ) {
        console.error("deliveryFeesToDestination failed: ", deliveryFeesToDestination);
        return;
    }
    const deliveryFeesToDestinationInDestinationAHNative = deliveryFeesToDestination.value.value[0].fun.value
    console.log(`deliveryFees ${plan.destinationAH.name} to ${plan.destination.name} [${plan.destinationAH.native_symbol}]: `, deliveryFeesToDestinationInDestinationAHNative);

    const destinationWeight =
        await plan.destination.api.apis.XcmPaymentApi.query_xcm_weight(messageToDestination);
    if (!destinationWeight.success) {
        console.error("destinationWeight failed: ", destinationWeight);
        return;
    }
    console.log("API: destinationWeight: ", destinationWeight.value);

    // Remote fees are only execution.
    const resultDestinationFeesInDestinationRelayNative =
        await plan.destination.api.apis.XcmPaymentApi.query_weight_to_asset_fee(
            destinationWeight.value,
            XcmVersionedAssetId.V5(plan.destinationAH.native_from_sibling),
        );

    if (!resultDestinationFeesInDestinationRelayNative.success) {
        console.error(`destinationFeesInDestinationRelayNative failed: `, resultDestinationFeesInDestinationRelayNative);
        return;
    }
    const destinationFeesInDestinationRelayNative = resultDestinationFeesInDestinationRelayNative.value;

    console.log(`API: localExecutionFees (virtual) [TEER]: `, localExecutionFees);
    console.log(`API: delivery fees to ${plan.sourceAH.name}         [TEER]: `, deliveryFeesToSourceAHInTeer);
    console.log(`API: ${plan.sourceAH.name} fees*                     [${plan.sourceAH.native_symbol}]: `, sourceAHFeesInNative.value);
    console.log(`API: delivery fees to ${plan.destinationAH.name}          [${plan.sourceAH.native_symbol}]: `, deliveryFeesToDestinationAHInSourceAHNative);
    console.log(`API: ${plan.destinationAH.name} fees*                     [${plan.destinationAH.native_symbol}]: `, destinationAHFeesInNative.value);
    console.log(`API: delivery fees to ${plan.destination.name}          [${plan.destinationAH.native_symbol}]: `, deliveryFeesToDestinationInDestinationAHNative);
    console.log(`API: ${plan.destination.name} fees                      [${plan.destinationAH.native_symbol}]: `, destinationFeesInDestinationRelayNative);

    console.log(`simulated rate as TEER per ${plan.sourceAH.native_symbol}: `, teerPerSourceAHNative, ` with TEER converted for fees: `, teerSpent, ` equal to fees in ${plan.sourceAH.native_symbol}: `, swapCreditEvent.value.value.amount_out);
    console.log(`simulated rate as ${plan.sourceAH.native_symbol} per ${plan.destinationAH.native_symbol}: `, sourceAHNativePerDestinationAHNative / 100, ` with ${plan.sourceAH.native_symbol} converted for fees: `, sourceAHNativeSpent, ` equal to fees in ${plan.destinationAH.native_symbol}: `, swapCreditEvent2.value.value.amount_out);


    const sourceAHFeesInTeer = BigInt(Math.round(Number(sourceAHFeesInNative.value + deliveryFeesToDestinationAHInSourceAHNative) * teerPerSourceAHNative * 1.1));
    console.log(`${plan.sourceAH.name} fees before swap (with margin*) [TEER]: `, sourceAHFeesInTeer);

    const destinationAHFeesInSourceAHNative = BigInt(Math.round(Number(destinationAHFeesInNative.value + deliveryFeesToDestinationInDestinationAHNative) * sourceAHNativePerDestinationAHNative * 1.1));
    console.log(`${plan.destinationAH.name} fees before swap (with margin*) [${plan.sourceAH.native_symbol}]: `, destinationAHFeesInSourceAHNative);

    const totalCallerFeesInTeer = localExecutionFees +
        deliveryFeesToSourceAHInTeer + sourceAHFeesInTeer +
        BigInt(Math.round(Number(destinationAHFeesInSourceAHNative + deliveryFeesToDestinationAHInSourceAHNative) * teerPerSourceAHNative +
            Number(destinationFeesInDestinationRelayNative + deliveryFeesToDestinationInDestinationAHNative) * teerPerSourceAHNative * sourceAHNativePerDestinationAHNative))
    console.log("to be paid by caller to cover everything [TEER]: ", Number(totalCallerFeesInTeer) / Number(TEER_UNITS));

    return [totalCallerFeesInTeer, sourceAHFeesInTeer, destinationAHFeesInSourceAHNative, destinationFeesInDestinationRelayNative + deliveryFeesToDestinationInDestinationAHNative];
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