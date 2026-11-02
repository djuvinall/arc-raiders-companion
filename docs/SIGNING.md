# Windows code signing

The installer isn't Authenticode-signed yet, so SmartScreen warns on the first install. In-app updates are checked with the updater key instead.

No option outside the Microsoft Store removes the warning on day one. Microsoft's guidance is that a signature builds reputation over time, and since 2024 that includes EV certificates. Self-signing doesn't help public users.

| Option | Cost | Who qualifies | SmartScreen | Publisher shown |
| --- | --- | --- | --- | --- |
| [Azure Artifact Signing](https://learn.microsoft.com/en-us/azure/artifact-signing/quickstart) | About $9.99 a month | Individuals in the US or Canada, with government ID | Warnings fade as reputation builds | Your legal name |
| [Certum Open Source](https://shop.certum.eu/open-source-code-signing-code.html) | From about €49 | Open-source developers, worldwide | Same | Your name |
| [SignPath Foundation](https://signpath.org/terms) | Free | Approved open-source projects with a verifiable reputation | Same | SignPath Foundation |
| [Microsoft Store (MSIX)](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options) | Free | Anyone | No warnings | Your Store name |

## Recommended: Azure Artifact Signing

It's the cheapest route that needs no hardware token and works from GitHub Actions, as long as you're an individual in the US or Canada.

1. Create an Azure subscription with a billing account of type Individual. Its legal name and address must match your ID.
2. Register the `Microsoft.CodeSigning` provider and create an Artifact Signing account.
3. Complete individual identity validation in the Azure portal. It uses a government ID and the Microsoft Authenticator app. Microsoft says organization checks take 1 to 20 business days, so allow time for this too.
4. Create a Public Trust certificate profile.

Then add the certificate details to the release workflow, along with a `signCommand` in `tauri.conf.json`. Sign every release with the same identity so reputation carries across versions.

Your legal name, city, state and country appear on the certificate.

## Microsoft Store

This is the only route with no warnings from the first download, but it's a different shipping model. The Store handles updates, so the GitHub updater wouldn't apply, and the app would need MSIX packaging and Store certification.
