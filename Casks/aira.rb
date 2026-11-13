cask "aira" do
  version "1.0.0"
  sha256 :no_check

  url "https://github.com/gilangggra/Aira/releases/download/v#{version}/Aira.zip"
  name "Aira"
  desc "Notch companion that keeps an eye on your AI coding agents"
  homepage "https://github.com/gilangggra/Aira"

  # Only the Mac's tags (vX.Y.Z): windows-v…, linux-v… and android-v… are other apps.
  livecheck do
    url :url
    regex(/^v?(\d+(?:\.\d+)+)$/i)
    strategy :github_releases
  end

  # Aira updates itself from the notch: brew leaves the installed app alone.
  auto_updates true
  depends_on macos: :sequoia

  app "Aira.app"

  uninstall quit: "fr.louisraille.NotchBuddy"

  zap trash: [
    "~/Library/Application Support/Aira",
    "~/Library/Application Support/Coucou",
    "~/Library/Application Support/NotchBuddy",
    "~/Library/Preferences/fr.louisraille.NotchBuddy.plist",
  ]
end
