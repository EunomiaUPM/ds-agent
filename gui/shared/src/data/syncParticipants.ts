import { customInstance } from "shared/src/data/orval-mutator";

export const syncParticipants = async (): Promise<void> => {
  try {
    await customInstance(`/mates/sync`, { method: "POST" });
  } catch (err) {
    console.error(err);
  }
};
