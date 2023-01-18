const BASE_URI = "wry://localhost";

export function refresh_sites() {
  const origin = document.getElementById("origin").value;

  // update audio src
  const audio_link =
    "https://dict.youdao.com/dictvoice?audio=" + origin + "&le=ja";
  const audio_node = document.getElementById("youdao_audio");
  audio_node.src = audio_link;
  const audio_a_node = document.getElementById("youdao_audio_link");
  audio_a_node.href = audio_link;

  fetch(
    BASE_URI + "/sites.json" + "?" + new URLSearchParams({ origin: origin })
  )
    // `invoke` returns a Promise
    .then((response) => response.json())
    .then((data) => {
      console.log(data);

      const origin_sites = document.getElementById("sites");
      if (!!origin_sites) {
        origin_sites.remove();
      }
      const new_sites = document.createElement("div");
      new_sites.id = "sites";
      const ul_node = document.createElement("ul");

      for (var sites of data) {
        const node = document.createElement("li");
        const a_node = document.createElement("a");
        a_node.href = sites.url;
        a_node.target = "_blank";
        a_node.textContent = sites.brand;
        node.appendChild(a_node);
        ul_node.appendChild(node);
      }
      new_sites.appendChild(ul_node);
      document.body.appendChild(new_sites);
    });
}

export function translate_text() {
  const origin = document.getElementById("origin").value;
  fetch(
    BASE_URI + "/translate.json" + "?" + new URLSearchParams({ origin: origin })
  )
    // `invoke` returns a Promise
    .then((response) => response.json())
    .then((data) => {
      console.log(data);
      document.getElementById("translated").value = data.translated;
      refresh_sites();
    });
}

export function load_content() {
  fetch(BASE_URI + "/data.json")
    // `invoke` returns a Promise
    .then((response) => response.json())
    .then((data) => {
      console.log(data);
      document.getElementById("origin").value = data.origin;
      document.getElementById("translated").value = data.translated;
      refresh_sites();
    });
}

export async function init() {
  load_content();

  document
    .querySelector("#translate_button")
    .addEventListener("click", translate_text);
}
