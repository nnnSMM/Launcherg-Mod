import {
  commandGetNotRegisterdDetailElementIds,
  commandCreateElementDetails,
} from "@/lib/command";
import { scrapeSql } from "@/lib/scrapeSql";

let registerDetailsInFlight: Promise<void> | null = null;

const registerCollectionElementDetailsOnce = async () => {
  const ids = await commandGetNotRegisterdDetailElementIds();
  if (!ids.length) {
    return;
  }

  const query = `select gamelist.id, gamelist.furigana, gamelist.sellday, gamelist.okazu, brandlist.brandname, brandlist.brandfurigana from gamelist inner join brandlist on brandlist.id = gamelist.brandname where gamelist.id IN (${ids.join(
    ", "
  )});`;
  const rows = await scrapeSql(query, 6);
  await commandCreateElementDetails(rows.map(mapRowToElementDetail));
};

export const registerCollectionElementDetails = async () => {
  if (registerDetailsInFlight) {
    return registerDetailsInFlight;
  }

  registerDetailsInFlight = registerCollectionElementDetailsOnce().finally(
    () => {
      registerDetailsInFlight = null;
    },
  );
  return registerDetailsInFlight;
};

export const __resetRegisterCollectionElementDetailsForTest = () => {
  registerDetailsInFlight = null;
};

export const mapRowToElementDetail = (row: string[]) => ({
  collectionElementId: +row[0],
  gamenameRuby: row[1],
  sellday: row[2],
  isNukige: row[3].includes("t"),
  brandname: row[4],
  brandnameRuby: row[5],
});
