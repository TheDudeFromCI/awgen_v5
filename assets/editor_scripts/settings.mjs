/**
 * A small class to manage the project settings.
 */
export class ProjectSettings {
  #projectName = null;
  #projectVersion = null;

  /**
   * This constructor initializes the project settings with the given game API.
   * Upon initialization, it will query the engine for the current project
   * settings to cache them.
   * @param {GameAPI} api The game API to use for communication with the engine.
   */
  constructor(api) {
    api.once("engineStarted", (response) => {
      this.#projectName = response.projectName;
      this.#projectVersion = response.projectVersion;
    });
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
    COMMAND({
      command: "setProjectName",
      name: this.#projectName,
    });
  }

  /**
   * This function sets the version of the project. Calling this method will
   * update the project settings in the engine.
   * @param {string} version The version of the project.
   */
  setVersion(version) {
    this.#projectVersion = version;
    COMMAND({
      command: "setProjectVersion",
      version: this.#projectVersion,
    });
  }
}
