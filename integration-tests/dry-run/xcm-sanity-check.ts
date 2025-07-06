// sanity check for a zombienet, chopsticks or live chain setup requires a running IK-KAH chopsticks

// `pah` and 'kah' are the names we gave to `bun papi add`.
import {
    itk, // bun papi add itk -w http://localhost:8001
    itp, // bun papi add itp -w http://localhost:8003
    kah, // bun papi add kah -w http://localhost:8000
    pah, // bun papi add pah -w http://localhost:8002
    dot,
    ksm,
    XcmV5Junction,
    XcmV5Junctions,
    XcmV5NetworkId,
    XcmVersionedLocation,
} from "@polkadot-api/descriptors";
import {
    createClient,
} from "polkadot-api";
import {getWsProvider} from "polkadot-api/ws-provider/node";
import {withPolkadotSdkCompat} from "polkadot-api/polkadot-sdk-compat";

const LIVE: number = 0;
const CHOPSTICKS: number = 1;
const ZOMBIENET: number = 2;

const ENDPOINTS = CHOPSTICKS;

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

const KAH_WS_URL = ENDPOINTS === LIVE
    ? "wss://sys.ibp.network/asset-hub-kusama"
    : ENDPOINTS === CHOPSTICKS
        ? "ws://localhost:8000"
        : "ws://localhost:9010";
const IK_WS_URL = ENDPOINTS === LIVE
    ? "wss://kusama.api.integritee.network"
    : ENDPOINTS === CHOPSTICKS
        ? "ws://localhost:8001"
        : "ws://localhost:9144";
const PAH_WS_URL = ENDPOINTS === LIVE
    ? "wss://sys.ibp.network/asset-hub-polkadot"
    : ENDPOINTS === CHOPSTICKS
        ? "ws://localhost:8002"
        : "ws://localhost:9910";
const IP_WS_URL = ENDPOINTS === LIVE
    ? "wss://polkadot.api.integritee.network"
    : ENDPOINTS === CHOPSTICKS
        ? "ws://localhost:8003"
        : "ws://localhost:9244";
const KSM_WS_URL = ENDPOINTS === LIVE
    ? "wss://rpc.ibp.network/kusama"
    : ENDPOINTS === CHOPSTICKS
        ? "skipped"
        : "ws://localhost:9945";
const DOT_WS_URL = ENDPOINTS === LIVE
    ? "wss://rpc.ibp.network/polkadot"
    : ENDPOINTS === CHOPSTICKS
        ? "skipped"
        : "ws://localhost:9942";

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

const ksmClient = createClient(
    withPolkadotSdkCompat(getWsProvider(KSM_WS_URL)),
);
const ksmApi = ksmClient.getTypedApi(ksm);

const dotClient = createClient(
    withPolkadotSdkCompat(getWsProvider(DOT_WS_URL)),
);
const dotApi = dotClient.getTypedApi(dot);

main();

async function main() {
    switch (ENDPOINTS) {
        case LIVE:
            console.log("Running against live chains...");
            break;
        case CHOPSTICKS:
            console.log("Running against chopsticks...");
            break;
        case ZOMBIENET:
            console.log("Running against zombienet...");
            break;
        default:
            throw new Error(`Unknown ENDPOINTS value: ${ENDPOINTS}`);
    }
    if (ENDPOINTS !== CHOPSTICKS) {
        await checkHrmpChannels()
    }
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
        checkLocationBalanceOn(itkApi, XcmVersionedLocation.V5(TEER_FROM_SELF), 5n * TEER_UNITS, "ITK Sovereign Local on ITK [TEER]"),
        checkLocationBalanceOn(kahApi, XcmVersionedLocation.V5(ITK_FROM_SIBLING), 5n * KSM_UNITS, "ITK Sovereign on KAH [KSM]"),
        checkLocationBalanceOn(pahApi, XcmVersionedLocation.V5(ITK_FROM_COUSIN), 5n * DOT_UNITS, "ITK Sovereign on PAH [DOT]"),
        // ITP sovereign
        checkLocationBalanceOn(itpApi, XcmVersionedLocation.V5(TEER_FROM_SELF), 5n * TEER_UNITS, "ITP Sovereign Local on ITK [TEER]"),
        checkLocationBalanceOn(pahApi, XcmVersionedLocation.V5(ITP_FROM_SIBLING), 5n * DOT_UNITS, "ITP Sovereign on PAH [DOT]"),
        checkLocationBalanceOn(kahApi, XcmVersionedLocation.V5(ITP_FROM_COUSIN), 5n * KSM_UNITS, "ITP Sovereign on KAH [KSM]"),
        // AH sovereign
        checkLocationBalanceOn(itkApi, XcmVersionedLocation.V5(KAH_FROM_SIBLING), 5n * TEER_UNITS, "KAH Sovereign on ITK [TEER]"),
        checkLocationBalanceOn(itpApi, XcmVersionedLocation.V5(PAH_FROM_SIBLING), 5n * TEER_UNITS, "PAH Sovereign on ITP [TEER]"),
    ])
}

async function checkLocationBalanceOn(api: any, location: XcmVersionedLocation, expectedBalance: bigint, label: string) {
    try {
        const accountIdResult = await api.apis.LocationToAccountApi.convert_location(location);
        if (accountIdResult.success) {
            await checkAccountIdBalanceOn(api, accountIdResult.value, expectedBalance, label);
        } else {
            console.log(`❌ ${label} failed to convert location to account ID:`, accountIdResult);
        }
    } catch (error) {
        console.log(`❌ ${label} error:`, error?.message ?? error);
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
    try {
        const outAmount = await api.apis.AssetConversionApi.quote_price_tokens_for_exact_tokens(inLocation, outLocation, inAmount, true);
        const actualPrice = Number(outAmount) / Number(inAmount) / scalingFactor;
        if (actualPrice > refPrice / toleranceFactor && actualPrice < refPrice * toleranceFactor) {
            console.log(`✅ ${label} price ${actualPrice} within expected tolerance from ${refPrice} (factor ${toleranceFactor})`);
        } else {
            console.log(`❌ ${label} price ${actualPrice} violates tolerance from ${refPrice} (factor ${toleranceFactor})`);
        }
    } catch (error) {
        console.log(`❌ ${label} error:`, error?.message ?? error);
    }
}
