## Obfuscation on Modrinth

To ensure the safety of all Modrinth users, projects may only be uploaded with obfuscated code under specific circumstances.</br>

- Projects that use third-party code or assets required by law or licensing restrictions to remain obfuscated.
- Projects where the obfuscation demonstrably benefits end users in a way critical to its functionality or safety.
- Projects where obfuscation is required to prevent the bypass of critical authorization checks.

### Uploading your project to Modrinth without obfuscation

If your project does NOT qualify for one of the above exemptions, we ask that you:

- Remove the use of obfuscation from your project.
- Remove all versions containing obfuscated code from your project before resubmission.

### Uploading your qualifying project with obfuscation

If you believe your project should be permitted to use obfuscation, you must follow all steps when resubmitting your project:

- Provide sufficient evidence that your project falls into one of the allowed exemptions.
- Ensure that our moderation team is able to verify the safety of your source code and that compiled outputs match provided sources.

We understand that you may not want to publish the source code for this project, so you are welcome to share it privately to the [Modrinth Content Moderation Team](https://github.com/ModrinthModeration) on GitHub.</br>
Please be aware that you will be required to maintain up-to-date sources indefinitely, your project may be rejected without warning if our moderation team is unable to confirm that any version of your project originates from verifiably safe sources.

We strongly recommend that you use an automated build system to ensure that your project's outputs verifiably originate from the provided source code and are always identical to the files uploaded to Modrinth.</br>

Alternatively, please ensure your provided sources contain:

- Instructions to reliably produce both non-obfuscated and obfuscated builds within a fresh environment.
- No non-deterministic obfuscation methods.

Finally, please note that we broadly discourage the use of obfuscation and advise against it unless absolutely required, and that the review of your project will require significantly more time when obfuscation is used.
