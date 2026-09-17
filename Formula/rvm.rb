class Rvm < Formula
  desc "Git-like version control for resumes with LaTeX compilation, AI tailoring, and job tracking"
  homepage "https://github.com/rvm-project/rvm"
  url "https://github.com/rvm-project/rvm/archive/refs/tags/v0.0.2.tar.gz"
  # sha256 "UPDATE_WITH_ACTUAL_SHA256_AFTER_RELEASE"
  license "MIT"

  depends_on "rust" => :build
  depends_on "tectonic"  # LaTeX compilation engine - auto-installed with `brew install rvm`

  def install
    system "cargo", "install", *std_cargo_args(path: "crates/rvm-cli")
  end

  test do
    system "#{bin}/rvm", "--version"

    # Test workspace initialization
    mkdir "test_workspace" do
      system "#{bin}/rvm", "init"
      assert_predicate Pathname.pwd/".rvm/HEAD", :exist?
    end
  end
end
