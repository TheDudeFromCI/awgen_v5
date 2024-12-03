import { EditorAPI } from "./api.mjs"

let editor = new EditorAPI();

editor.once("engine_started", async () => {
  print("AwgenScript Editor engine started.");
  editor.settings.setName("Default Project Template");
  editor.settings.setVersion("0.0.1");
});

await editor.run();
