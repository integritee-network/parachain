// sanity check for a zombienet, chopsticks or live chain setup

// `pah` and 'kah' are the names we gave to `bun papi add`.
import {
    itk, // bun papi add itk -w http://localhost:8001 | bun papi add itk -w http://localhost:9144
    itp, // bun papi add itp -w http://localhost:8003 | bun papi add itp -w http://localhost:9244
    kah, // bun papi add kah -w http://localhost:8000 | bun papi add kah -w http://localhost:9010
    pah, // bun papi add pah -w http://localhost:8002 | bun papi add pah -w http://localhost:9910
    dot,
    ksm,
    XcmV5Junction,
    XcmV5Junctions,
    XcmV5NetworkId,
    XcmVersionedLocation, XcmV3JunctionBodyId, XcmV2JunctionBodyPart
} from "@polkadot-api/descriptors";
import {
    createClient, FixedSizeBinary
} from "polkadot-api";
import {getWsProvider} from "polkadot-api/ws-provider/node";
import {withPolkadotSdkCompat} from "polkadot-api/polkadot-sdk-compat";

const LIVE: number = 0;

// for chopsticks we assume this setup:
// Kusama and Polkadot side must be run separately:
// npx @acala-network/chopsticks@latest xcm --p=./configs/kusama-asset-hub.yml --p=./configs/integritee-kusama.yml
// should be ports 8000 and 8001 respectively.
// npx @acala-network/chopsticks@latest xcm --p=./configs/polkadot-asset-hub.yml --p=./configs/integritee-polkadot.yml
// should be ports 8002 and 8003 respectively.
const CHOPSTICKS: number = 1;

// for zombienet we assume the setup described in ../bridges/README.md:
const ZOMBIENET: number = 2;

// use this constant to select your endpoint set
const ENDPOINTS = ZOMBIENET;

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

// Useful constants.
const KAH_PARA_ID = 1000;
const PAH_PARA_ID = 1000;
const IK_PARA_ID = 2015;
const IP_PARA_ID = 2039;

const TEER_UNITS = 1_000_000_000_000n;
const KSM_UNITS = 1_000_000_000_000n;
const DOT_UNITS = 10_000_000_000n;

const KSM_FROM_COUSIN_PARACHAINS = {
    parents: 2,
    interior: XcmV5Junctions.X1(XcmV5Junction.GlobalConsensus(XcmV5NetworkId.Kusama())),
};
const DOT_FROM_COUSIN_PARACHAINS = {
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
const KAH_FROM_SIBLING = {
    parents: 1,
    interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(KAH_PARA_ID)),
};
const PAH_FROM_SIBLING = {
    parents: 1,
    interior: XcmV5Junctions.X1(XcmV5Junction.Parachain(PAH_PARA_ID)),
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
const ALICE_PUB = FixedSizeBinary.fromHex("0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"); // well-known alice account
const ALICE_LOCAL = {
    parents: 0,
    interior: XcmV5Junctions.X1(XcmV5Junction.AccountId32({id: ALICE_PUB}))
}
const palletId = Buffer.from("modlpy/trsry", "utf8"); // 8 bytes
const padded = Buffer.concat([palletId, Buffer.alloc(32 - palletId.length, 0)]);
const treasuryAccount = FixedSizeBinary.fromHex(padded.toHex());
const TREASURY_LOCAL = {
    parents: 0,
    interior: XcmV5Junctions.X1(XcmV5Junction.AccountId32({id: treasuryAccount}))
}

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
    console.log("checking (foreign) asset balances")
    await Promise.all([
        printLocationForeignAssetBalanceOn(kahApi, XcmVersionedLocation.V5(ITK_FROM_SIBLING), XcmVersionedLocation.V5(ITK_FROM_SIBLING), "ITK Sovereign on KAH [TEER]"),
        printLocationForeignAssetBalanceOn(pahApi, XcmVersionedLocation.V5(ITP_FROM_SIBLING), XcmVersionedLocation.V5(ITP_FROM_SIBLING), "ITP Sovereign on PAH [TEER]"),
        printLocationForeignAssetBalanceOn(pahApi, XcmVersionedLocation.V5(ITK_FROM_COUSIN), XcmVersionedLocation.V5(KSM_FROM_COUSIN_PARACHAINS), "ITK Sovereign on PAH [KSM]"),
        printLocationForeignAssetBalanceOn(kahApi, XcmVersionedLocation.V5(ITP_FROM_COUSIN), XcmVersionedLocation.V5(DOT_FROM_COUSIN_PARACHAINS), "ITP Sovereign on KAH [DOT]"),
        printLocationAssetBalanceOn(itkApi, XcmVersionedLocation.V5(ITP_FROM_COUSIN), 0, "ITP Sovereign on ITK [KSM]"),
        printLocationAssetBalanceOn(itpApi, XcmVersionedLocation.V5(ITK_FROM_COUSIN), 0, "ITK Sovereign on ITP [DOT]"),

    ])
    if ((ENDPOINTS === CHOPSTICKS) || (ENDPOINTS === ZOMBIENET)) {
        // Assume alice is porting TEER to test, so check relevant balances additionally
        console.log("checking (foreign) asset balances for well-known keys")
        await Promise.all([
            printLocationAssetBalanceOn(itpApi, XcmVersionedLocation.V5(ALICE_LOCAL), 0, "Alice on ITP [DOT]"),
            printLocationAssetBalanceOn(itkApi, XcmVersionedLocation.V5(ALICE_LOCAL), 0, "Alice on ITK [KSM]"),
            checkLocationBalanceOn(itpApi, XcmVersionedLocation.V5(ALICE_LOCAL), 0n, "Alice on ITP [TEER]"),
            checkLocationBalanceOn(itkApi, XcmVersionedLocation.V5(ALICE_LOCAL), 0n, "Alice on ITK [TEER]"),
            printLocationAssetBalanceOn(itpApi, XcmVersionedLocation.V5(TREASURY_LOCAL), 0, "Treasury on ITP [DOT]"),
            printLocationAssetBalanceOn(itkApi, XcmVersionedLocation.V5(TREASURY_LOCAL), 0, "Treasury on ITK [KSM]"),
            checkLocationBalanceOn(itpApi, XcmVersionedLocation.V5(TREASURY_LOCAL), 0n, "Treasury on ITP [TEER]"),
            checkLocationBalanceOn(itkApi, XcmVersionedLocation.V5(TREASURY_LOCAL), 0n, "Treasury on ITK [TEER]"),
            printLocationForeignAssetBalanceOn(kahApi, XcmVersionedLocation.V5(ALICE_LOCAL), XcmVersionedLocation.V5(ITK_FROM_SIBLING), "Alice on KAH [TEER]"),
            printLocationForeignAssetBalanceOn(pahApi, XcmVersionedLocation.V5(ALICE_LOCAL), XcmVersionedLocation.V5(ITP_FROM_SIBLING), "Alice on PAH [TEER]"),
            printLocationForeignAssetBalanceOn(kahApi, XcmVersionedLocation.V5(ALICE_LOCAL), XcmVersionedLocation.V5(KSM_FROM_SIBLING_PARACHAINS), "Alice on KAH [KSM]"),
            printLocationForeignAssetBalanceOn(pahApi, XcmVersionedLocation.V5(ALICE_LOCAL), XcmVersionedLocation.V5(DOT_FROM_SIBLING_PARACHAINS), "Alice on PAH [DOT]"),
            printLocationForeignAssetBalanceOn(kahApi, XcmVersionedLocation.V5(ALICE_LOCAL), XcmVersionedLocation.V5(DOT_FROM_COUSIN_PARACHAINS), "Alice on KAH [DOT]"),
            printLocationForeignAssetBalanceOn(pahApi, XcmVersionedLocation.V5(ALICE_LOCAL), XcmVersionedLocation.V5(KSM_FROM_COUSIN_PARACHAINS), "Alice on PAH [KSM]"),
        ])
    }
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

async function printLocationForeignAssetBalanceOn(api: any, account_location: XcmVersionedLocation, asset_location: XcmVersionedLocation, label: string) {
    try {
        const accountIdResult = await api.apis.LocationToAccountApi.convert_location(account_location);
        if (accountIdResult.success) {
            await printAccountIdForeignAssetBalanceOn(api, accountIdResult.value, asset_location, label);
        } else {
            console.log(`❌ ${label} failed to convert location to account ID:`, accountIdResult);
        }
    } catch (error) {
        console.log(`❌ ${label} error:`, error?.message ?? error);
    }

}

async function printAccountIdForeignAssetBalanceOn(api: any, accountId: string, location: XcmVersionedLocation, label: string) {
    try {
        const assetBalanceResult = await api.query.ForeignAssets.Account.getValue(location.value, accountId);
        console.log(`  ${label} (${accountId}) balance: ${assetBalanceResult?.balance ?? 0n}`);
    } catch (error) {
        console.log(`❌ ${label} (${accountId}) error:`, error?.message ?? error);
    }
}

async function printLocationAssetBalanceOn(api: any, account_location: XcmVersionedLocation, asset_id: number, label: string) {
    try {
        const accountIdResult = await api.apis.LocationToAccountApi.convert_location(account_location);
        if (accountIdResult.success) {
            await printAccountIdAssetBalanceOn(api, accountIdResult.value, asset_id, label);
        } else {
            console.log(`❌ ${label} failed to convert location to account ID:`, accountIdResult);
        }
    } catch (error) {
        console.log(`❌ ${label} error:`, error?.message ?? error);
    }

}

async function printAccountIdAssetBalanceOn(api: any, accountId: string, asset_id: number, label: string) {
    try {
        const assetBalanceResult = await api.query.Assets.Account.getValue(asset_id, accountId);
        console.log(`  ${label} (${accountId}) balance: ${assetBalanceResult?.balance ?? 0n}`);
    } catch (error) {
        console.log(`❌ ${label} (${accountId}) error:`, error?.message ?? error);
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
        checkAssetConversionOn(pahApi, KSM_FROM_COUSIN_PARACHAINS, DOT_FROM_SIBLING_PARACHAINS, referenceAmount, usdPerDot / usdPerKsm, Number(KSM_UNITS) / Number(DOT_UNITS), toleranceFactor, "KSM per DOT on PAH"),
        checkAssetConversionOn(kahApi, ITK_FROM_SIBLING, KSM_FROM_SIBLING_PARACHAINS, referenceAmount, usdPerKsm / usdPerTeer, Number(TEER_UNITS) / Number(KSM_UNITS), toleranceFactor, "TEER PER KSM on KAH"),
        checkAssetConversionOn(kahApi, DOT_FROM_COUSIN_PARACHAINS, KSM_FROM_SIBLING_PARACHAINS, referenceAmount, usdPerKsm / usdPerDot, Number(DOT_UNITS) / Number(KSM_UNITS), toleranceFactor, "DOT per KSM on KAH"),
        checkAssetConversionOn(pahApi, ITP_FROM_SIBLING, DOT_FROM_SIBLING_PARACHAINS, referenceAmount, usdPerDot / usdPerTeer, Number(TEER_UNITS) / Number(DOT_UNITS), toleranceFactor, "TEER PER DOT on PAH"),
        checkAssetConversionOn(itpApi, '{ "Native" }', '{ "WithId": 0 }', referenceAmount, usdPerDot / usdPerTeer, Number(TEER_UNITS) / Number(DOT_UNITS), toleranceFactor, "TEER PER DOT on ITP"),
        checkAssetConversionOn(itkApi, '{ "Native" }', '{ "WithId": 0 }', referenceAmount, usdPerKsm / usdPerTeer, Number(TEER_UNITS) / Number(KSM_UNITS), toleranceFactor, "TEER PER KSM on ITK"),
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
