import { GameAPI } from "./api.mjs"

print("Starting AwgenScript Editor engine.")

let api = new GameAPI();

api.once("engine_started", async () => {
  print("AwgenScript Editor engine started.");

  print("Updating project settings...");
  await api.setProjectSettings("Default Project Template", "1.0.0");
  print("Project settings updated.");

  let [name, version] = await api.getProjectSettings();
  print(`Project name: ${name}, version: ${version}`);
});

await api.run();
