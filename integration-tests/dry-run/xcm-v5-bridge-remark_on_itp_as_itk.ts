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

// if false, we assum zombienet
const CHOPSTICKS: boolean = true;

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

const IP_FROM_PAH = {
    parents: 1,
    interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(IP_PARA_ID)),
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
const DOT_UNITS = 10_000_000_000n;

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
const DOT_FROM_SIBLING_PARACHAINS = {
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

const itpClient = createClient(
    withPolkadotSdkCompat(getWsProvider(IP_WS_URL)),
);
const itpApi = itpClient.getTypedApi(itp);

// The whole execution of the script.
main();

// We'll teleport KSM from Asset Hub to People.
// Using the XcmPaymentApi and DryRunApi, we'll estimate the XCM fees accurately.
async function main() {
    // The amount of TEER we wish to teleport besides paying fees.
    const transferAmount = 0n;
    // We overestimate both local and remote fees, these will be adjusted by the dry run below.
    const localFeesHighEstimate = 1n * TEER_UNITS; // we're root locally and don't pay fees for execution, but for delivery we do.
    const remote1FeesHighEstimateTeer = 10n * TEER_UNITS;
    const remote2FeesHighEstimateKsm = 1n * KSM_UNITS / 10n;
    const remote3FeesHighEstimateDot = 10n * DOT_UNITS;

    if (CHOPSTICKS) {
        const stx = await itkApi.tx.System.remark_with_event({remark: Binary.fromText("Let's trigger state migration")})
        const signer = getAliceSigner();
        await stx.signAndSubmit(signer);
        const stx2 = await itpApi.tx.System.remark_with_event({remark: Binary.fromText("Let's trigger state migration")})
        await stx2.signAndSubmit(signer);
        console.log("triggered state migrations if necessary. waiting for a bit....")
        // wait for chopsticks api's to catch up
        await new Promise(resolve => setTimeout(resolve, 5000));
    }

    const referenceAmountTeer = 1000000000n;
    const remote2FeesHighEstimateTeerConverted = await kahApi.apis.AssetConversionApi.quote_price_tokens_for_exact_tokens(TEER_FROM_SIBLING, KSM_FROM_KUSAMA_PARACHAINS, referenceAmountTeer, true);
    const teerPerKsm = Number(remote2FeesHighEstimateTeerConverted) / Number(referenceAmountTeer)
    console.log("Current AssetConversion quote for remote1 account: out: ", remote2FeesHighEstimateTeerConverted, " in ", referenceAmountTeer, " TEER. price: ", teerPerKsm, " TEER per KSM");

    const referenceAmountKsm = 1000000000n;
    const remote3FeesHighEstimateKsmConverted = await pahApi.apis.AssetConversionApi.quote_price_tokens_for_exact_tokens(KSM_FROM_POLKADOT_PARACHAINS, DOT_FROM_SIBLING_PARACHAINS, referenceAmountKsm, true);
    const ksmPerDot = Number(remote3FeesHighEstimateKsmConverted) / Number(referenceAmountKsm)
    console.log("Current AssetConversion quote for remote2 account: out: ", remote3FeesHighEstimateKsmConverted, " in ", referenceAmountKsm, " KSM. price: ", ksmPerDot / 100.0, " KSM per DOT");

    // We create a tentative XCM, one with the high estimates for fees.
    const tentativeXcm = await createXcm(
        transferAmount,
        localFeesHighEstimate,
        remote1FeesHighEstimateTeer,
        BigInt(Math.round(Number(remote2FeesHighEstimateTeerConverted) * 1.5)),
        remote2FeesHighEstimateKsm,
        remote3FeesHighEstimateDot
    );
    console.dir(stringify(tentativeXcm));

    // We can now query the root account on the source chain.
    const rootAccountLocal = await itkApi.apis.LocationToAccountApi.convert_location(XcmVersionedLocation.V5(TEER_FROM_SELF))
    if (!rootAccountLocal.success) {
        console.error("Failed to get root account on source chain: ", rootAccountLocal);
        await itkClient.destroy();
        await kahClient.destroy();
        await pahClient.destroy();
        await itpClient.destroy();
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
        await itpClient.destroy();
        return;
    }
    const sovereignAccountInfoRemote2 = await itkApi.query.System.Account.getValue(rootAccountLocal.value)
    console.log("Sovereign account of ITK on remote2 ITP chain: ", rootAccountRemote2, " with AccountInfo ", sovereignAccountInfoRemote2);

    const weightRes = await itkApi.apis.XcmPaymentApi.query_xcm_weight(tentativeXcm);
    if (!weightRes.success) {
        console.error("Failed to get weight for tentative XCM: ", weightRes);
        await itkClient.destroy();
        await kahClient.destroy();
        await pahClient.destroy();
        await itpClient.destroy();
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
    const [localFeesEstimate, remote1FeesEstimate, remote2FeesEstimateKsm, remote3FeesEstimateDot] =
        (await estimateFees(tentativeXcm))!;

    console.log("Local fees estimate [TEER]: ", localFeesEstimate);
    console.log("Remote 1 fees estimate [TEER]: ", remote1FeesEstimate);
    console.log("Remote 2 fees estimate [KSM]: ", remote2FeesEstimateKsm);
    console.log("Remote 3 fees estimate [DOT]: ", remote3FeesEstimateDot);
    // With these estimates, we can create the final XCM to execute.
    const xcm = await createXcm(
        transferAmount,
        localFeesEstimate,
        remote1FeesEstimate,
        BigInt(Math.round(Number(remote2FeesEstimateKsm) * teerPerKsm * 1.5)),
        remote2FeesEstimateKsm,
        remote3FeesEstimateDot
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
        console.log("Await System.Remarked event on destination chain...this can take a few minutes")
        if (CHOPSTICKS) {
            console.error("Chopsticks doesn't emulate the bridge yet. the following will stall after KAH")
        }
        const startTime = Date.now();
        await Promise.all([
            itkApi.event.PolkadotXcm.Sent.watch()
                .pipe(take(1))
                .forEach((event) => {
                    const elapsedTime = Date.now() - startTime;
                    console.log(`ITK Event received after ${elapsedTime}ms: PolkadotXcm.Sent id `, event.payload.message_id.asHex(), " to ", event.payload.destination);
                }),
            kahApi.event.AssetConversion.SwapCreditExecuted.watch()
                .pipe(take(1))
                .forEach((event) => {
                    const elapsedTime = Date.now() - startTime;
                    console.log(`KAH Event received after ${elapsedTime}ms: AssetConversion.SwapCreditExecuted `, event.payload);
                }),
            kahApi.event.PolkadotXcm.Sent.watch()
                .pipe(take(1))
                .forEach((event) => {
                    const elapsedTime = Date.now() - startTime;
                    console.log(`KAH Event received after ${elapsedTime}ms: PolkadotXcm.Sent id `, event.payload.message_id.asHex(), " to ", event.payload.destination);
                }),
            pahApi.event.PolkadotXcm.Sent.watch()
                .pipe(take(1))
                .forEach((event) => {
                    const elapsedTime = Date.now() - startTime;
                    console.log(`PAH Event received after ${elapsedTime}ms: PolkadotXcm.Sent id `, event.payload.message_id.asHex(), " to ", event.payload.destination);
                }),
            itpApi.event.System.Remarked.watch()
                .pipe(take(1))
                .forEach((event) => {
                    const elapsedTime = Date.now() - startTime;
                    console.log(`ITP Event received after ${elapsedTime}ms: System.Remarked from `, event.payload.sender, " with hash ", event.payload.hash.asHex());
                })
        ])
    }
    await itkClient.destroy();
    await kahClient.destroy();
    await pahClient.destroy();
    await itpClient.destroy();
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
    remote3FeesDot: bigint,
): Promise<XcmVersionedXcm> {
    const executeOnItp = itpApi.tx.System.remark_with_event({remark: Binary.fromText("Hello Integritee on Polkadot")})
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
    const dotForRemote3Fees = {
        id: DOT_FROM_POLKADOT_PARACHAINS,
        fun: XcmV3MultiassetFungibility.Fungible(remote3FeesDot),
    };
    const teerForRemote1Filter = XcmV5AssetFilter.Definite([teerForRemote1Total]);
    const teerToSwapOnRemote1Filter = XcmV5AssetFilter.Definite([teerToSwapOnRemote1]);
    const ksmForRemote2Filter = XcmV5AssetFilter.Definite([ksmForRemote2Fees]);
    const dotForRemote3Filter = XcmV5AssetFilter.Definite([dotForRemote3Fees]);

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
                // to execute on KAH
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
                        // to execute on PAH
                        XcmV5Instruction.SetAppendix([
                            XcmV5Instruction.RefundSurplus(),
                            XcmV5Instruction.DepositAsset({
                                assets: XcmV5AssetFilter.Wild(XcmV5WildAsset.All()),
                                beneficiary: TEER_FROM_COUSIN,
                            })
                        ]),
                        // as a shortcut, let's use DOT we already have on sovereign account
                        XcmV5Instruction.WithdrawAsset([dotForRemote3Fees]),
                        XcmV5Instruction.InitiateTransfer({
                            destination: IP_FROM_PAH,
                            preserve_origin: true,
                            remote_fees: Enum("ReserveDeposit", dotForRemote3Filter),
                            assets: [],
                            remote_xcm: [
                                // to execute on IP
                                XcmV5Instruction.SetAppendix([
                                    XcmV5Instruction.RefundSurplus(),
                                    XcmV5Instruction.DepositAsset({
                                        assets: XcmV5AssetFilter.Wild(XcmV5WildAsset.All()),
                                        beneficiary: TEER_FROM_COUSIN,
                                    })
                                ]),
                                XcmV5Instruction.Transact({
                                    origin_kind: XcmV2OriginKind.SovereignAccount(),
                                    call: await executeOnItp.getEncodedData(),
                                }),
                            ],
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
): Promise<[bigint, bigint, bigint, bigint] | undefined> {
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
    const deliveryFeesToRemote1Teer = deliveryFees.value.value[0].fun.value;
    console.log("deliveryFees to remote1 [TEER]: ", deliveryFeesToRemote1Teer);

    // Local fees for execution (which is virtual as root won't pay execution).
    const localExecutionFees = executionFees.value;

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

    // We get the delivery fees using the size of the forwarded xcm.
    const deliveryFeesToRemote2 = await kahApi.apis.XcmPaymentApi.query_delivery_fees(
        XcmVersionedLocation.V5(PAH_FROM_KAH),
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
    console.log("deliveryFees KAH to PAH [KSM]: ", deliveryFeesToRemote2Ksm);

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

    // XCM execution might result in multiple messages being sent.
    // That's why we need to search for our message in the `forwarded_xcms` array.
    const [_dummy3, messages3] = remote2DryRunResult.value.forwarded_xcms.find(
        ([location, _]) =>
            location.type === "V5" &&
            location.value.parents === 1 &&
            location.value.interior.type === "X1" &&
            location.value.interior.value.type === "Parachain" &&
            location.value.interior.value.value === IP_PARA_ID,
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

    // Now we dry run on the destination.
    const remote3DryRunResult = await itpApi.apis.DryRunApi.dry_run_xcm(
        // XCM origin has to be KAH. It will then AliasOrigin into IK upon execution
        // see runtime patch to allow this: https://github.com/polkadot-fellows/runtimes/compare/main...encointer:runtimes:ab/trusted-aliaser-patch
        XcmVersionedLocation.V5({
            parents: 1,
            interior: XcmV5Junctions.X1(
                XcmV5Junction.Parachain(PAH_PARA_ID)
            ),
        }),
        messageToItp,
    );
    if (
        !remote3DryRunResult.success ||
        remote3DryRunResult.value.execution_result.type !== "Complete"
    ) {
        console.error("remote3DryRunResult failed: ", remote3DryRunResult);
        return;
    }
    console.log("remote3DryRunResult: ", remote3DryRunResult.value);

    // We get the delivery fees using the size of the forwarded xcm.
    const deliveryFeesToRemote3 = await pahApi.apis.XcmPaymentApi.query_delivery_fees(
        XcmVersionedLocation.V5(IP_FROM_PAH),
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
    console.log("deliveryFees PAH to ITP [DOT]: ", deliveryFeesToRemote3Dot);

    const remote3Weight =
        await itpApi.apis.XcmPaymentApi.query_xcm_weight(messageToItp);
    if (!remote3Weight.success) {
        console.error("remote3Weight failed: ", remote3Weight);
        return;
    }
    console.log("API: remote3Weight: ", remote3Weight.value);

    // Remote fees are only execution.
    const resultRemote3FeesInDot =
        await itpApi.apis.XcmPaymentApi.query_weight_to_asset_fee(
            remote3Weight.value,
            XcmVersionedAssetId.V5(DOT_FROM_POLKADOT_PARACHAINS),
        );

    if (!resultRemote3FeesInDot.success) {
        console.error("remote3FeesInDot failed: ", resultRemote3FeesInDot);
        return;
    }
    const remote3FeesInDot = 4580824760n //TODO: weight_2_fee on ITP seems off:  resultRemote3FeesInDot.value. See: https://github.com/integritee-network/parachain/issues/329

    console.log("API: localExecutionFees (virtual) [TEER]: ", localExecutionFees);
    console.log("API: deliveryFeesToRemote1Teer    [TEER]: ", deliveryFeesToRemote1Teer);
    console.log("API: remote1FeesInKsm*            [KSM] : ", remote1FeesInKsm.value);
    console.log("API: deliveryFeesToRemote2Ksm     [KSM] : ", deliveryFeesToRemote2Ksm);
    console.log("API: remote2FeesInDot*            [DOT] : ", remote2FeesInDot.value);
    console.log("API: deliveryFeesToRemote3Dot     [DOT] : ", deliveryFeesToRemote3Dot);
    console.log("API: remote3FeesInDot             [DOT] : ", remote3FeesInDot);

    console.log("simulated rate as TEER per KSM: ", teerPerKsm, " with TEER converted for fees: ", teerSpent, " equal to fees in KSM: ", swapCreditEvent.value.value.amount_out);
    console.log("simulated rate as KSM per DOT: ", ksmPerDot / 100, " with KSM converted for fees: ", ksmSpent, " equal to fees in DOT: ", swapCreditEvent2.value.value.amount_out);


    const remote1FeesInTeer = deliveryFeesToRemote1Teer + BigInt(Math.round(Number(remote1FeesInKsm.value) * teerPerKsm * 1.1));
    console.log("remote1FeesInTeer (with margin*): ", remote1FeesInTeer);

    const remote2FeesInKsm = deliveryFeesToRemote2Ksm + BigInt(Math.round(Number(remote2FeesInDot.value) * ksmPerDot * 1.1));
    console.log("remote2FeesInKsm (with margin*): ", remote2FeesInKsm);

    const totalCallerFeesInTeer = localExecutionFees +
        deliveryFeesToRemote1Teer + remote1FeesInTeer +
        BigInt(Math.round(Number(remote2FeesInKsm + deliveryFeesToRemote2Ksm) * teerPerKsm +
            Number(remote3FeesInDot + deliveryFeesToRemote3Dot) * teerPerKsm * ksmPerDot))
    console.log("to be paid by caller to cover everything: ", Number(totalCallerFeesInTeer) / Number(TEER_UNITS), " TEER");

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