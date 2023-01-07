import { emit, listen } from "@tauri-apps/api/event";

// access the pre-bundled global API functions
const { invoke } = window.__TAURI__.tauri;

export function refresh_sites() {
  const origin = document.getElementById("origin").value;

  // update audio src
  const audio_link =
    "https://dict.youdao.com/dictvoice?audio=" + origin + "&le=ja";
  const audio_node = document.getElementById("youdao_audio");
  audio_node.src = audio_link;
  const audio_a_node = document.getElementById("youdao_audio_link");
  audio_a_node.href = audio_link;

  invoke("web_get_translate_sites", { origin: origin })
    // `invoke` returns a Promise
    .then((response) => {
      console.log(response);

      const origin_sites = document.getElementById("sites");
      if (!!origin_sites) {
        origin_sites.remove();
      }
      const new_sites = document.createElement("div");
      new_sites.id = "sites";
      const ul_node = document.createElement("ul");

      for (sites of response) {
        const node = document.createElement("li");
        const a_node = document.createElement("a");
        a_node.href = sites[1];
        a_node.target = "_blank";
        a_node.textContent = sites[0];
        node.appendChild(a_node);
        ul_node.appendChild(node);
      }
      new_sites.appendChild(ul_node);
      document.body.appendChild(new_sites);
    });
}

export function translate_text() {
  const origin = document.getElementById("origin").value;
  invoke("web_translate", { origin: origin })
    // `invoke` returns a Promise
    .then((response) => {
      document.getElementById("translated").value = response;
      refresh_sites();
    });
}

export function load_content() {
  invoke("web_get_origin")
    // `invoke` returns a Promise
    .then((response) => {
      document.getElementById("origin").value = response;
      refresh_sites();
    });
  invoke("web_get_translated")
    // `invoke` returns a Promise
    .then((response) => {
      document.getElementById("translated").value = response;
    });
}

export async function init() {
  load_content();

  const unlisten = await listen("reload_content", (event) => {
    console.log("received event: ");
    console.log(event);
    load_content();
  });
}
