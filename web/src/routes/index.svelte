<script lang="ts">
  import { onMount } from "svelte";
  import wasm_script from "../../../pkg/asteroids_3d_lib.js?url";
  import wasm_mod from "../../../pkg/asteroids_3d_lib_bg.wasm?url";
  import loadScript from "$lib/loadScript";

  let bytesLoaded = 0;
  let bytesTotal = 1;
  let errorMessage = '';

  function prettyBytes(bytes: number) {
    if (bytes < 1024 ** 2) {
      return Math.round(bytes / 1024) + " KB";
    }
    return Math.round(bytes / 1024 ** 2) + " MB";
  }

  async function load() {
    await loadScript(wasm_script);

    // https://stackoverflow.com/a/65529994/4471524
    async function load() {
      const response = await fetch(wasm_mod);

      // Note - If you are compressing your .wasm file the Content-Length will be incorrect
      // One workaround is to use a custom http header to manually specify the uncompressed size
      const contentLength = response.headers.get("Content-Length");

      const total = parseInt(contentLength, 10) || 1;
      let loaded = 0;

      function progressHandler(loaded, total) {
        bytesLoaded = loaded;
        bytesTotal = total;
      }

      var res = new Response(
        new ReadableStream(
          {
            async start(controller) {
              var reader = response.body.getReader();
              for (;;) {
                var { done, value } = await reader.read();

                if (done) {
                  if (total > 1) progressHandler(total, total);
                  break;
                }

                loaded += value.byteLength;
                progressHandler(loaded, total);
                controller.enqueue(value);
              }
              controller.close();
            },
          },
          {
            status: response.status,
            statusText: response.statusText,
          }
        )
      );

      // Make sure to copy the headers!
      // Wasm is very picky with it's headers and it will fail to compile if they are not
      // specified correctly.
      for (const pair of response.headers.entries()) {
        res.headers.set(pair[0], pair[1]);
      }
      return res;
    }

    wasm_bindgen(load());
  }

  onMount(() => {
	  load().catch(e => {
		  console.error(e);
		  errorMessage = e.toString();
	  })
  });
</script>

<svelte:head>
  <title>Home</title>
</svelte:head>

<section>
  <div class="w-full h-screen overflow-hidden">
    <canvas class="w-0 h-0" id="can" />
    <div class="w-full h-full flex justify-center px-10 flex-col">
      <span class="text-2xl">
        {prettyBytes(bytesLoaded)} of {bytesTotal <= 1
          ? "..."
          : prettyBytes(bytesTotal)}
      </span>
      <div class="w-full mb-2 bg-blue-900 rounded-xl">
        <div
          style={`width: ${(bytesLoaded / bytesTotal) * 100}%`}
          class="bg-blue-500 py-5 rounded-xl"
        />
      </div>
	  <p class="text-red-300">{errorMessage}</p>
    </div>
  </div>
</section>
