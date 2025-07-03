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
    dot,
    ksm,
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

// We're running against chopsticks with wasm-override to get XCMv5 support.
// `npx @acala-network/chopsticks@latest xcm --p=kusama-asset-hub --p=./configs/integritee-kusama.yml`
// const KAH_WS_URL = "ws://localhost:8000";
// const IK_WS_URL = "ws://localhost:8001";
// const PAH_WS_URL = "ws://localhost:8002";
// const IP_WS_URL = "ws://localhost:8003";

// if running against the bridge zombienet instead, use these:
const KAH_WS_URL = "ws://localhost:9010";
const IK_WS_URL = "ws://localhost:9144";
const PAH_WS_URL = "ws://localhost:9910";
const IP_WS_URL = "ws://localhost:9244";
const KSM_WS_URL = "ws://localhost:9945";
const DOT_WS_URL = "ws://localhost:9942";

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
const DOT_FROM_KUSAMA_PARACHAINS = {
    parents: 2,
    interior: XcmV5Junctions.X1(XcmV5Junction.GlobalConsensus(XcmV5NetworkId.Polkadot())),
};
const DOT_FROM_SIBLING_PARACHAINS = {
    parents: 1,
    interior: XcmV5Junctions.Here(),
};
const KSM_FROM_SIBLING_PARACHAINS = {
    parents: 1,
    interior: XcmV5Junctions.Here(),
};
const TEER_FROM_SELF = {
    parents: 0,
    interior: XcmV5Junctions.Here(),
};
const ITK_FROM_SIBLING = {
    parents: 1,
    interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(IK_PARA_ID)),
};
const KAH_FROM_SIBLING = {
    parents: 1,
    interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(KAH_PARA_ID)),
};
const PAH_FROM_SIBLING = {
    parents: 1,
    interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(PAH_PARA_ID)),
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
    interior: XcmV5Junctions.X2([XcmV5Junction.GlobalConsensus(XcmV5NetworkId.Kusama()), XcmV5Junction.Parachain(IP_PARA_ID)]),
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

const ksmClient = createClient(
    withPolkadotSdkCompat(getWsProvider(KSM_WS_URL)),
);
const ksmApi = ksmClient.getTypedApi(ksm);

const dotClient = createClient(
    withPolkadotSdkCompat(getWsProvider(DOT_WS_URL)),
);
const dotApi = dotClient.getTypedApi(dot);

// The whole execution of the script.
main();

// We'll teleport KSM from Asset Hub to People.
// Using the XcmPaymentApi and DryRunApi, we'll estimate the XCM fees accurately.
async function main() {
    await checkHrmpChannels()
    await checkBalances()
    await checkAssetConversions();
    await itkClient.destroy();
    await kahClient.destroy();
    await pahClient.destroy();
    await itpClient.destroy();
    await dotClient.destroy();
    await ksmClient.destroy();
}

async function checkHrmpChannels() {
    console.log("Checking HRMP channels on KSM...");
    const ksmActualChannels = await ksmApi.query.Hrmp.HrmpChannels.getEntries();
    const ksmExpectedChannels: [number, number][] = [
        [1000, 1002],
        [1002, 1000],
        [1000, 2015],
        [2015, 1000],
    ];
    checkHrmpChannelsResults(ksmActualChannels, ksmExpectedChannels);
    console.log("Checking HRMP channels on DOT...");
    const dotActualChannels = await dotApi.query.Hrmp.HrmpChannels.getEntries();
    const dotExpectedChannels: [number, number][] = [
        [1000, 1002],
        [1002, 1000],
        [1000, 2039],
        [2039, 1000],
    ];
    checkHrmpChannelsResults(dotActualChannels, dotExpectedChannels);
}

function checkHrmpChannelsResults(channels: any[], expectedChannels: [number, number][]) {
    for (const [from, to] of expectedChannels) {
        const found = channels.some(({keyArgs}) =>
            Array.isArray(keyArgs) &&
            keyArgs.length === 1 &&
            typeof keyArgs[0] === "object" &&
            keyArgs[0] !== null &&
            "sender" in keyArgs[0] &&
            "recipient" in keyArgs[0] &&
            Number(keyArgs[0].sender) === from &&
            Number(keyArgs[0].recipient) === to
        );
        if (found) {
            console.log(`✅ Channel [${from}, ${to}] exists`);
        } else {
            console.log(`❌ Channel [${from}, ${to}] missing`);
        }
    }
}


async function checkBalances() {
    console.log("checking sovereign balances")
    await Promise.all([
        // ITK sovereign
        checkLocationBalanceOn(itkApi, XcmVersionedLocation.V5(TEER_FROM_SELF), 10_000_000_000_000n, "ITK Sovereign Local on ITK [TEER]"),
        checkLocationBalanceOn(kahApi, XcmVersionedLocation.V5(ITK_FROM_SIBLING), 10_000_000_000_000n, "ITK Sovereign on KAH [KSM]"),
        checkLocationBalanceOn(pahApi, XcmVersionedLocation.V5(ITK_FROM_COUSIN), 10_0_000_000_000n, "ITK Sovereign on PAH [DOT]"),
        // ITP sovereign
        checkLocationBalanceOn(itpApi, XcmVersionedLocation.V5(TEER_FROM_SELF), 10_000_000_000_000n, "ITP Sovereign Local on ITK [TEER]"),
        checkLocationBalanceOn(pahApi, XcmVersionedLocation.V5(ITP_FROM_SIBLING), 10_0_000_000_000n, "ITP Sovereign on PAH [DOT]"),
        checkLocationBalanceOn(kahApi, XcmVersionedLocation.V5(ITP_FROM_COUSIN), 10_000_000_000_000n, "ITP Sovereign on KAH [KSM]"),
        // AH sovereign
        checkLocationBalanceOn(itkApi, XcmVersionedLocation.V5(KAH_FROM_SIBLING), 10_000_000_000_000n, "KAH Sovereign on ITK [TEER]"),
        checkLocationBalanceOn(itpApi, XcmVersionedLocation.V5(PAH_FROM_SIBLING), 10_000_000_000_000n, "PAH Sovereign on ITP [TEER]"),
    ])
}

async function checkLocationBalanceOn(api: any, location: XcmVersionedLocation, expectedBalance: bigint, label: string) {
    const accountIdResult = await api.apis.LocationToAccountApi.convert_location(location);
    if (accountIdResult.success) {
        await checkAccountIdBalanceOn(api, accountIdResult.value, expectedBalance, label);
    } else {
        console.log(`❌ ${label} failed to convert location to account ID:`, accountIdResult);
    }
}

async function checkAccountIdBalanceOn(api: any, accountId: string, expectedBalance: bigint, label: string) {
    const accountInfoResult = await api.query.System.Account.getValue(accountId);
    if (accountInfoResult.data) {
        const balance = accountInfoResult.data.free || 0n;
        if (balance >= expectedBalance) {
            console.log(`✅ ${label} (${accountId}) balance: ${balance} is at least ${expectedBalance}`);
        } else {
            console.log(`❌ ${label} (${accountId}) balance: ${balance} is less than expected ${expectedBalance}`);
        }
    } else {
        console.log(`❌ ${label} Account not found`);
    }
}


async function checkAssetConversions() {
    console.log("Checking asset conversions on various DEXes...");
    // lazily update these prices not to deviate too much from actual market prices
    const usdPerDot = 4.0
    const usdPerKsm = 10.0
    const usdPerTeer = 0.20
    const referenceAmount = 1_000_000n; // should be small enough not to care about 10 or 12 decimals
    const toleranceFactor = 10;
    console.log(`reference prices: USD per DOT: ${usdPerDot}, USD per KSM: ${usdPerKsm}, USD per TEER: ${usdPerTeer}`);
    await Promise.all([
        checkAssetConversionOn(pahApi, KSM_FROM_POLKADOT_PARACHAINS, DOT_FROM_SIBLING_PARACHAINS, referenceAmount, usdPerDot / usdPerKsm, Number(KSM_UNITS) / Number(DOT_UNITS), toleranceFactor, "KSM per DOT on PAH"),
        checkAssetConversionOn(kahApi, ITK_FROM_SIBLING, KSM_FROM_KUSAMA_PARACHAINS, referenceAmount, usdPerKsm / usdPerTeer, Number(TEER_UNITS) / Number(KSM_UNITS), toleranceFactor, "TEER PER KSM on KAH"),
        checkAssetConversionOn(kahApi, DOT_FROM_KUSAMA_PARACHAINS, KSM_FROM_SIBLING_PARACHAINS, referenceAmount, usdPerKsm / usdPerDot, Number(DOT_UNITS) / Number(KSM_UNITS), toleranceFactor, "DOT per KSM on KAH"),
        checkAssetConversionOn(pahApi, ITP_FROM_SIBLING, DOT_FROM_POLKADOT_PARACHAINS, referenceAmount, usdPerDot / usdPerTeer, Number(TEER_UNITS) / Number(DOT_UNITS), toleranceFactor, "TEER PER DOT on PAH"),
    ]);
}

async function checkAssetConversionOn(api: any, inLocation: any, outLocation: any, inAmount: bigint, refPrice: number, scalingFactor: number, toleranceFactor: number = 2, label: string) {
    const outAmount = await api.apis.AssetConversionApi.quote_price_tokens_for_exact_tokens(inLocation, outLocation, inAmount, true);
    const actualPrice = Number(outAmount) / Number(inAmount) / scalingFactor;
    if (actualPrice > refPrice / toleranceFactor && actualPrice < refPrice * toleranceFactor) {
        console.log(`✅ ${label} price ${actualPrice} within expected tolerance from ${refPrice} (factor ${toleranceFactor})`);
    } else {
        console.log(`❌ ${label} price ${actualPrice} violates tolerance from ${refPrice} (factor ${toleranceFactor})`);
    }
}
