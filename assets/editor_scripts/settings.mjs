/**
 * A small class to manage the project settings.
 */
export class ProjectSettings {
  #projectName = null;
  #projectVersion = null;
  #api = null;

  /**
   * This constructor initializes the project settings with the given game API.
   * Upon initialization, it will query the engine for the current project
   * settings to cache them.
   * @param {GameAPI} api The game API to use for communication with the engine.
   */
  constructor(api) {
    this.#api = api;

    api.query({ query: "project_settings" }).then((response) => {
      if (!this.#projectName) this.#projectName = response.name;
      if (!this.#projectVersion) this.#projectVersion = response.version;
    });
  }

  /**
   * This function sends a command to the native layer of the engine to update
   * the project settings with the given name and version. This is a private
   * function and should not be called directly.
   *
   * If the project version or name is not set, it will wait for the next
   * "project_settings" event from the engine to cache the values before sending
   * the update command.
   */
  #update() {
    if (!this.#projectName || !this.#projectVersion) {
      this.#api.once("project_settings", () => {
        COMMAND({
          command: "set_project_settings",
          name: this.#projectName,
          version: this.#projectVersion
        });
      });
    } else {
      COMMAND({
        command: "set_project_settings",
        name: this.#projectName,
        version: this.#projectVersion
      });
    }
  }

  /**
   * This function returns the name of the project.
   * @returns {string} The name of the project.
   */
  getName() {
    return this.#projectName;
  }

  /**
   * This function returns the version of the project.
   * @returns {string} The version of the project.
   */
  getVersion() {
    return this.#projectVersion;
  }

  /**
   * This function sets the name of the project. Calling this method will update
   * the project settings in the engine.
   * @param {string} name The name of the project.
   */
  setName(name) {
    this.#projectName = name;
    this.#update();
  }

  /**
   * This function sets the version of the project. Calling this method will
   * update the project settings in the engine.
   * @param {string} version The version of the project.
   */
  setVersion(version) {
    this.#projectVersion = version;
    this.#update();
  }
}
