import { createPreferencePage, createItems } from "../helpers";
import { invoke } from "@tauri-apps/api/core";

export const appleIdPage = createPreferencePage(
  "apple-id",
  "Apple ID",
  [
    createItems.select(
      "anisette-server",
      "Anisette Server",
      [
        {
          label: "jkcoxson (Windows ADI)",
          value: "ani.jkcoxson.com",
        },
        { label: "SideStore (New)", value: "new.ani.sidestore.io" },
        { label: "jkcoxson", value: "mac-ani.jkcoxson.com" },
        {
          label: "SideStore (Windows ADI)",
          value: "win.ani.sidestore.io",
        },
        { label: "Lill q", value: "anisette.lillq.me" },
        { label: "iLoader", value: "ani.iloader.app" },
        {
          label: "Custom",
          value: "other",
          default: "ani.yourserver.com",
        },
      ],
      "The remote anisette server used. Change this if you are having issues logging in.",
      "ani.jkcoxson.com"
    ),
    {
      id: "apple-id-email",
      name: "Apple ID",
      description: "The apple ID email you are currently logged in with.",
      type: "info",
      defaultValue: async () => {
        const appleId = await invoke<string>("get_apple_email");
        return appleId || "Not logged in";
      },
    },
    createItems.button(
      "reset-anisette",
      "Reset Anisette",
      "Remove all anisette data (will require 2fa again)",
      "danger",
      "soft",
      async () => {
        await invoke("reset_anisette");
      }
    ),
    createItems.button(
      "reset-credentials",
      "Reset Saved Credentials",
      "Remove saved Apple ID credentials and anisette data",
      "danger",
      "soft",
      async () => {
        await invoke("delete_stored_credentials");
        await invoke("reset_anisette");
      }
    ),
  ],
  {
    description: "Manage your Apple ID authentication",
    category: "apple",
  }
);
