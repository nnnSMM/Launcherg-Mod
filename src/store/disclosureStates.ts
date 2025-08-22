import { localStorageWritable } from "@/lib/utils";

type DisclosureStates = {
  launcherg: boolean;
  help: boolean;
  memo: boolean;
};

const initialStates: DisclosureStates = {
  launcherg: true,
  help: true,
  memo: true,
};

export const disclosureStates = localStorageWritable<DisclosureStates>(
  "disclosure-states",
  initialStates
);
