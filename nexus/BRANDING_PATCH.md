# BRANDING_PATCH

Danh sách file mà nhánh `nexus` sửa so với upstream vì lý do thương hiệu.
Mỗi mục là đường dẫn file chính xác trong dấu backtick (không dùng glob/thư mục).

Giá trị: sản phẩm **Nexus PDF**, bộ **NexusOffice**, nhà phát hành **SonnyTech**, app id
`vn.sonnytech.nexus.pdf`, tài sản phát hành `NexusPDF-*`, binary cài đặt `nexus-pdf`/`nexus-pdf-cli`
(MSI, zip portable, deb/rpm/AppImage/tar, zip CLI macOS; tên build Cargo vẫn là `pdfcraft`/`pdfcraft-cli`,
bundle macOS `Nexus PDF.app` với executable `NexusPDF`). Thư mục người dùng: `NexusPDF` (macOS/Windows),
`nexuspdf` (XDG), `NexusPDFData` (portable) — một nguồn duy nhất `crates/engine/src/nexus.rs`. Không có
khoảng trắng và dạng XDG là chữ thường của nó, để `eframe::storage_dir` (macOS giữ tên nhưng đổi khoảng
trắng, Linux bỏ khoảng trắng và viết thường) và các đường dẫn tự dựng (khôi phục, clipboard, Digital IDs,
socket một phiên bản) cùng ra **một** thư mục trên mọi nền tảng, như `PdfCraft`/`pdfcraft` của upstream
(review T10 vòng 1; test `settings_dir_survives_eframe_normalisation`).

Nguồn của bản build (About, `crates/engine/src/nexus.rs` `SOURCE_URL`, MCP `websiteUrl`, CLI `--version`, metadata gói, README):
`https://github.com/sonnytech-vn/pdfcraft` (tổ chức sẽ được tạo và repo chuyển sang; GitHub chuyển hướng URL cũ). Chưa đổi
(chức năng, không phải nguồn): kiểm tra cập nhật (`apps/pdfcraft/src/updates.rs` gọi `api.github.com/repos/nhdai111983/pdfcraft`,
`crates/ui-egui/src/updates.rs` `RELEASES_PAGE`), remote git, workflow đồng bộ upstream, `GITHUB_REPOSITORY`/zsync của bản
phát hành. Sau khi chuyển repo, cần đổi hai hằng của kiểm tra cập nhật cùng lúc.

Giữ nguyên định danh máy (không hiện cho người dùng): ProgId `Sonnytech.NexusPDF.Document`, khoá registry
`Software\Sonnytech\Nexus PDF\…` (Capabilities, Shortcuts) và giá trị `RegisteredApplications\NexusPDF`, `UpgradeCode`
và GUID component WiX, app id `vn.sonnytech.nexus.pdf` (chữ thường), thư mục người dùng `NexusPDF`/`nexuspdf`/`NexusPDFData`,
`serverInfo.name = "pdfcraft"`, tên crate, biến `PDFCRAFT_*`, email maintainer `nhdai111983@users.noreply.github.com`.

Đổi thương hiệu 2026-10-23: Sonnytech → SonnyTech, NexusOffices → NexusOffice (Nexus PDF và các tên binary/tài sản
không đổi). `ATTRIBUTION.toml` đổi tác giả thành "SonnyTech (NexusOffice)" và cập nhật sha256 của hai file SVG đã sửa
chú thích; `ATTRIBUTION.md` sinh lại bằng `cargo xtask assets --write`.

Không đổi (spec §4.4): tên crate `pdfcraft-*` (và `short_name` trong `xtask/src/layers.rs`), biến
`PDFCRAFT_*`, URI `pdfcraft://doc/…` (tài nguyên MCP), `postMessage` `pdfcraft:*` của bản web, khoá
eframe storage `pdfcraft`/`printcraft`, MCP `serverInfo.name = "pdfcraft"` (định danh máy client cấu
hình theo), khoá i18n tiếng Anh trong 22 catalog `.tsv` (không sửa catalog), khoá định dạng ghi vào
PDF (`/PCUndrawn`, `/PCHelv`, dấu nội dung `q %PdfCraft`), log target `pdfcraft*` và tên file log
`pdfcraft.log`, tên thread/thư mục tạm `pdfcraft-*`, id DOM `pdfcraft`, khoá plist
`PdfCraftVersion`/`PdfCraftBuildCommit`, thư mục model OCR `share/pdfcraft/models` cho FreeBSD/Flatpak (chưa phát hành; Linux deb/rpm/tar/AppImage
dùng `share/nexus-pdf/models`),
Id phần tử WiX (`PdfcraftApp`…; chỉ là định danh trong MSI, không hiển thị).

Cơ chế chính:
- `crates/ui-egui/src/i18n/brand.rs` (file mới) — `pub fn brand(s) -> Cow<str>` thay
  `PdfCraft → Nexus PDF`, `ArtCraft → NexusOffice`. Áp **một lần** trong đường tra chuỗi:
  `LangInfo::catalog()` brand mọi bản dịch khi nạp catalog; tra chuỗi chưa dịch (English, hoặc
  catalog thiếu) brand khoá **chỉ khi khoá có trong một catalog** (`branded_key`), nên văn bản người
  dùng (tên file, giá trị trường, tác giả…) không bao giờ bị viết lại. `upstream_key` cho phép thông
  điệp engine đã đổi tại nguồn ("reinstall Nexus PDF") vẫn tìm được bản dịch upstream. Bất biến
  "không bao giờ áp cho văn bản người dùng" ghi trong doc comment của `tr`/`t`.
- `crates/engine/src/nexus.rs` (file mới) — hằng `APP_NAME`, `APP_ID`, `APP_BIN`, `CLI_BIN`,
  `SETTINGS_DIR(_XDG)`, `PORTABLE_DATA_DIR`, `PORTABLE_MARKER`, `DEFAULT_AUTHOR`, `SOURCE_URL`, câu ghi
  công (en/vi) và bản quyền upstream.
- `crates/ui-egui/src/nexus.rs` (file mới) — logo Nexus PDF **vẽ bằng code** (ô đỏ, trang giấy, chữ N
  nối hai nút vàng; không file ảnh, không gì của ArtCraft), các dòng About và thẻ đầu màn hình Home.

## File thương hiệu

| File | Lý do | Thay được bằng white-label? |
|---|---|---|
| `crates/ui-egui/src/i18n/mod.rs` | `mod brand; pub use brand::brand;` — `catalog()` brand bản dịch khi nạp; `tr()`/`trn()` brand khoá chưa dịch qua `branded_key`, tra khoá upstream qua `upstream_key`; `has()` nhất quán với `tr()`; doc comment bất biến; test `ui_literals_naming_upstream_are_branded_in_every_language`; test Arabic upstream so sánh với "Nexus PDF حول" | Có — hook "display name" trong i18n |
| `crates/ui-egui/src/widgets.rs` | Bỏ `artcraft_logo`/`artcraft_mark` (`include_bytes!` từ `docs/brand/`: logo ArtCraft không còn biên dịch vào binary); `community_links` không còn nút Discord đặc biệt | Có |
| `crates/ui-egui/src/dialogs.rs` | About: logo Nexus vẽ bằng code + tên `APP_NAME`; thay "Part of" + wordmark ArtCraft bằng `nexus::about_lines` (bộ NexusOffice/SonnyTech, câu ghi công, bản quyền upstream) | Có |
| `crates/ui-egui/src/home.rs` | Thẻ "Join the ArtCraft community"/Discord → `nexus::home_card` (logo, NexusOffice, link mã nguồn) | Có |
| `crates/ui-egui/src/chrome.rs` | Bỏ nút Discord trên thanh tiêu đề và bề rộng giữ chỗ cho nó | Có — `COMMUNITY_URL` tuỳ chọn |
| `crates/ui-egui/src/lib.rs` | `pub mod nexus;`; tiêu đề cửa sổ `"{tài liệu} — Nexus PDF"` | Có — `APP_NAME` |
| `crates/ui-egui/src/updates.rs` | `RELEASES_PAGE` → releases của fork | Có — URL qua biến build |
| `crates/ui-egui/src/portable.rs` | Marker `NexusPDF.portable`, thư mục `NexusPDFData` (từ `engine::nexus`) | Có |
| `crates/ui-egui/src/recovery.rs` | Thư mục khôi phục `NexusPDF/Recovery` / `nexuspdf/recovery` — không mở/xoá autosave của PdfCraft | Có |
| `crates/ui-egui/src/image_pick.rs` | Thư mục cache clipboard `NexusPDF/Clipboard` / `nexuspdf/clipboard`; tên file mặc định `Nexus PDF` | Có |
| `crates/ui-egui/src/sign_ui.rs` | Doc comment thư mục Digital IDs (theo thư mục khôi phục) | Đi cùng `recovery.rs` |
| `crates/ui-egui/src/web_print.rs` | `<title>` trang in của bản web | Có |
| `crates/ui-egui/tests/ui.rs` | Test bố cục thanh tab so với nút "Keyboard shortcuts" (không còn Discord); "Welcome to Nexus PDF" | Đi cùng `chrome.rs`/brand |
| `crates/ui-egui/tests/ui_group/links.rs` | Không còn Discord/ArtCraft; Home/About mở repo fork | Đi cùng `links.rs` |
| `crates/ui-egui/tests/ui_group/updates.rs` | URL releases fork; "Nexus PDF 99.0.0 is available" | Đi cùng `updates.rs` |
| `crates/ui-egui/tests/accessibility.rs` | Tiêu đề cửa sổ "— Nexus PDF" | Đi cùng `lib.rs` |
| `crates/ui-egui/tests/viewer.rs` | Tiêu đề cửa sổ "— Nexus PDF" | Đi cùng `lib.rs` |
| `crates/ui-egui/tests/pickers.rs` | "Welcome to Nexus PDF" | Đi cùng brand |
| `crates/ui-egui/tests/signing.rs` | Nhãn kho khoá Windows nêu Nexus PDF | Đi cùng brand |
| `crates/engine/src/lib.rs` | `pub mod nexus;`; hai thông điệp engine nêu `APP_NAME` | Có |
| `crates/engine/src/links.rs` | Bỏ `DISCORD`, `WEBSITE` (getartcraft); `APP_PAGE`/`GITHUB` → repo fork; `LINKS` chỉ còn "PdfCraft on GitHub" (khoá catalog → "Nexus PDF on GitHub"); test URL | Có |
| `crates/engine/src/commands.rs` | Bỏ lệnh `help.discord`, `help.website`, `help.app_page` (ArtCraft) | Có |
| `crates/create/src/lib.rs` | `/Producer (Nexus PDF)` ghi vào PDF mới | Có — tên app qua tham số |
| `crates/sign/src/pdf.rs` | `/Prop_Build /App /Name /NexusPDF` (một tên PDF, không khoảng trắng thoát) trong chữ ký và dấu thời gian; thông điệp kiểm tra chữ ký nêu Nexus PDF (khoá catalog upstream vẫn khớp nhờ `upstream_key`) | Có |
| `crates/sign/src/x509.rs` | Thông điệp phần mở rộng chứng chỉ nêu Nexus PDF | Có |
| `crates/sign/src/windows.rs` | Thông điệp bỏ qua khoá Windows nêu Nexus PDF | Có |
| `crates/preflight/src/icc.rs` | Mô tả hồ sơ ICC nhúng vào PDF: `sRGB (Nexus PDF)` | Có |
| `crates/ocr/src/lib.rs` | Lỗi thiếu model OCR: "reinstall Nexus PDF" (CLI/MCP/UI; bản dịch vẫn khớp nhờ `upstream_key`) ; `Models::dirs_beside_exe` thêm `../share/nexus-pdf/models` (gói Linux Nexus) + ca test bố cục gói | Có |
| `crates/edit/src/added.rs` | Lỗi thiếu phông Ả Rập/Hebrew; họ phông Type 3 ghi vào PDF `Nexus PDF Shaped Text` | Có |
| `crates/edit/src/text.rs` | Lỗi thiếu phông dự phòng nêu Nexus PDF | Có |
| `crates/annot/src/lib.rs` | Cảnh báo không vẽ lại được ghi chú nêu Nexus PDF | Có |
| `crates/redact/src/lib.rs` | Lỗi redact chữ dọc nêu Nexus PDF | Có |
| `crates/xfa/src/pdf.rs` | Ghi chú phông XFA nêu Nexus PDF | Có |
| `crates/automation/src/comments.rs` | Tác giả mặc định của ghi chú `Nexus PDF User` (ghi vào PDF) | Có |
| `crates/automation/src/mcp.rs` | `serverInfo.title = "Nexus PDF"`, `websiteUrl` repo fork, INSTRUCTIONS, tool meta `tool_search`/`tool_call`; **`serverInfo.name` giữ `"pdfcraft"`** | Có |
| `crates/automation/src/tools.rs` | Mô tả tool (link_open, doc_remove_marks, xfa, command_list, form_fill) nêu Nexus PDF | Có |
| `crates/automation/src/lib.rs` | `command_list`: nhãn lệnh tĩnh "About PdfCraft" → "About Nexus PDF" (`nexus_label`, không đụng nhãn có tên tài liệu) | Có |
| `apps/pdfcraft/src/main.rs` | Tiêu đề cửa sổ/eframe/menu macOS `APP_NAME`, app id `vn.sonnytech.nexus.pdf`, icon cửa sổ từ `nexus/assets/`, thư mục cấu hình/log `NexusPDF` (`nexuspdf` trên Linux), **không di trú/đụng thư mục PrintCraft/PdfCraft** (`NEXUS_KEEPS_UPSTREAM_FOLDERS`), `--version`/tiền tố panic `nexus-pdf`, gợi ý `~/.nexus-pdf-control.json`; test desktop file mới | Có |
| `apps/pdfcraft/src/settings.rs` | Thư mục XDG `nexuspdf` (+ test) | Có |
| `apps/pdfcraft/src/single_instance.rs` | Socket một phiên bản trong `%LOCALAPPDATA%\NexusPDF` / `$XDG_RUNTIME_DIR/nexuspdf`: **không bao giờ gửi file cho PdfCraft đang chạy** | Có |
| `apps/pdfcraft/src/updates.rs` | Kiểm tra cập nhật (chỉ khi người dùng bấm) hỏi releases của fork; User-Agent `NexusPDF/` | Có |
| `apps/pdfcraft/build.rs` | Resource Windows: icon Nexus, ProductName/FileDescription Nexus PDF, CompanyName SonnyTech, LegalCopyright (ghi công), OriginalFilename `nexus-pdf.exe` | Có |
| `apps/pdfcraft-cli/build.rs` | Resource Windows của CLI như trên, `nexus-pdf-cli.exe` | Có |
| `apps/pdfcraft-cli/src/main.rs` | USAGE, `--version` (không Discord/getartcraft), tiền tố lỗi `nexus-pdf-cli:`, gợi ý `nexus-pdf --control` | Có — tên binary qua biến build |
| `apps/pdfcraft-cli/tests/edit.rs` | Tiền tố lỗi `nexus-pdf-cli:` | Đi cùng `main.rs` |
| `apps/pdfcraft-cli/tests/help.rs` | USAGE `nexus-pdf-cli …` | Đi cùng `main.rs` |
| `apps/pdfcraft-cli/tests/root_arguments.rs` | Tiền tố lỗi `nexus-pdf-cli:` | Đi cùng `main.rs` |
| `apps/pdfcraft-cli/tests/text.rs` | Tiền tố lỗi `nexus-pdf-cli:` | Đi cùng `main.rs` |
| `apps/pdfcraft-cli/tests/version_resource.rs` | Giá trị VERSIONINFO mới (chỉ chạy trên Windows) | Đi cùng `build.rs` |
| `apps/pdfcraft-web/index.html` | `<title>Nexus PDF</title>` | Có |
| `apps/pdfcraft-web/src/main.rs` | Log console lỗi tải file | Có |
| `xtask/src/gates.rs` | Gate bundle id ↔ `APP_ID` theo `vn.sonnytech.nexus.pdf` | Đi cùng `main.rs`/plist |
| `packaging/windows/pdfcraft.wxs` | MSI: Name/Manufacturer **SonnyTech**, **UpgradeCode + mọi GUID component mới**, `Program Files\Nexus PDF`, `nexus-pdf.exe`/`nexus-pdf-cli.exe`, ProgId `Sonnytech.NexusPDF.Document`, `Applications\nexus-pdf.exe`, App Paths, `Software\Sonnytech\Nexus PDF\…`, `RegisteredApplications\NexusPDF`, menu chuột phải `NexusPDF.Edit`/`NexusPDF.CreatePdf` ("Edit with Nexus PDF"), shortcut `Nexus PDF.lnk`; ARP: bỏ getartcraft/Discord, About/Update → repo fork; NexusOffice: component `PdfcraftAttribution` (`ATTRIBUTION.md`, GUID tự sinh), thư mục `licenses\` + ComponentGroup `PdfcraftFontLicences` (`<Files Include="$(var.BinDir)\licenses\*">`, có điều kiện `FontLicences`); vòng sửa 2: chú thích — nhóm `FontLicences` phủ mọi văn bản giấy phép bên thứ ba (OFL, Lucide, Foxit, CMap Adobe) | Một phần (biến WiX) |
| `packaging/windows/installer-ui.wxs` | Nhãn checkbox "Edit with Nexus PDF" (các hộp thoại khác dùng `[ProductName]`) | Một phần |
| `packaging/windows/package.ps1` | `NexusPDF-*.msi/.zip`, exe đổi tên `nexus-pdf(-cli).exe`, icon Nexus; chép `ATTRIBUTION.md` vào thư mục stage (MSI) và zip portable; vòng sửa 1: dựng `licenses\` = OFL craft-fonts (`OFL-<family>.txt`) + `assets\fonts\OFL-*.txt` (tên duy nhất không phân biệt hoa thường; ném lỗi khi `CRAFT_FONTS_REQUIRED=1` mà không có OFL), truyền `-d FontLicences=yes|no` cho WiX, zip portable lấy OFL từ `licenses\`; vòng sửa 2: `Add-Licence` thêm `LICENSE-lucide.txt`, `LICENSE-FOXIT.txt`, `LICENSE-cmap-resources.txt` vào `licenses\` (bắt buộc); `FontLicences` nay phủ mọi văn bản giấy phép bên thứ ba (luôn `yes` với Nexus PDF) | Một phần |
| `packaging/windows/test-msi.ps1` | Smoke test MSI theo Manufacturer/khoá/nhãn mới; khẳng định `NOTICE`/`ATTRIBUTION.md` nằm trong `INSTALLFOLDER` và mọi `assets\fonts\OFL-*.txt` có trong `licenses\`; vòng sửa 2: so tên dài (`SHORT|long` → phần sau `|`) cho `INSTALLFOLDER` và `licenses\`, khẳng định thêm `LICENSE-lucide.txt`, `LICENSE-FOXIT.txt`, `LICENSE-cmap-resources.txt` | Đi cùng `pdfcraft.wxs` |
| `packaging/windows/sign.ps1` | Mô tả chữ ký `/d Nexus PDF`, `/du` repo fork | Một phần |
| `packaging/windows/portable.txt` | Hướng dẫn chế độ portable (nằm trong zip) | Không |
| `packaging/windows/security-audit.ps1` | Ví dụ tên file trong trợ giúp script | Không |
| `packaging/macos/Info.plist.in` | `Nexus PDF`, executable/icon `NexusPDF`, bundle id `vn.sonnytech.nexus.pdf`, copyright (ghi công) | Một phần |
| `packaging/macos/Credits.rtf` | About macOS: NexusOffice, ghi công, link fork (bỏ getartcraft/Discord) | Không |
| `packaging/macos/package.sh` | `Nexus PDF.app`, `NexusPDF-*.dmg`, `NexusPDF-cli-*.zip` chứa `nexus-pdf-cli`, icns Nexus, tên volume; không chép `dmg-layout.DS_Store`/nền (gắn tên PdfCraft); `copy_notices` chép `LICENSE-MIT`/`LICENSE-APACHE`/`NOTICE`/`ATTRIBUTION.md` và OFL của font repo vào `Contents/Resources` của app (trước khi ký) | Một phần |
| `packaging/linux/package.sh` | `APP_ID=vn.sonnytech.nexus.pdf`, `/usr/bin/nexus-pdf(-cli)`, `NexusPDF-*`, icon `nexus/assets/hicolor`, maintainer SonnyTech, zsync trỏ fork; model OCR ở `usr/share/nexus-pdf/models` (không đụng `/usr/share/pdfcraft` của gói PdfCraft đã cài); AppImage giữ `usr/share/doc/nexus-pdf` (giấy phép, `NOTICE`, `ATTRIBUTION.md`, OFL) thay vì bỏ ra ngoài | Một phần |
| `packaging/linux/nfpm.yaml` | Gói `nexus-pdf`, vendor SonnyTech, homepage fork, binary, ghi công | Một phần |
| `packaging/env.sh` | `copy_docs` = README/LICENSE/COPYRIGHT tuỳ có + OFL craft-fonts + `copy_notices` mới **bắt buộc** (`LICENSE-MIT`/`LICENSE-APACHE`/`NOTICE` — Apache-2.0 §4(d) — /`ATTRIBUTION.md`, README Nexus trỏ tới) + `assets/fonts/OFL-*.txt` (Inter, JetBrains Mono, Dancing Script nhúng trong binary; báo lỗi khi trùng tên); mọi `cp` có `|| return 1`; `copy_font_licences` báo lỗi khi `CRAFT_FONTS_REQUIRED=1` mà không có OFL. Dùng cho deb/AppImage/tar, zip CLI, zip web (và FreeBSD của upstream); vòng sửa 2: `copy_notices` chép thêm (bắt buộc) `assets/icons/LICENSE-lucide.txt` → `LICENSE-lucide.txt` (ISC + MIT Feather), `vendor/hayro-interpret/assets/LICENSE_FOXIT` → `LICENSE-FOXIT.txt` (font Foxit BSD-3), `assets/licenses/LICENSE-cmap-resources.txt` (CMap Adobe BSD-3) vào thư mục doc/`Contents/Resources`; cập nhật chú thích `copy_font_licences` | Nên PR upstream (gói upstream thiếu ATTRIBUTION.md/OFL của font repo) |
| `packaging/web/package.sh` | Zip `NexusPDF-web-<ver>.zip` | Một phần |
| `packaging/web/README.md` | Được đóng gói thành `HOSTING.md` trong zip web | Không |
| `README.md` | README Nexus PDF ngắn (vi trước, có English), câu ghi công "Nexus PDF được phát triển dựa trên PdfCraft của ArtCraft (MIT/Apache-2.0)" chỉ bằng chữ (không logo/Discord/getartcraft); `copy_docs` đóng gói nó | Không (README của fork) |
| `Cargo.toml` | `[workspace.package] repository` → `https://github.com/sonnytech-vn/pdfcraft` (brand-check `brand-url`; metadata, không code nào đọc `CARGO_PKG_REPOSITORY`) | Có — metadata của fork |
| `ATTRIBUTION.toml` | 14 mục `[[asset]]` cho icon Nexus (`nexus/assets/**`, SonnyTech, CC0-1.0, có sha256), nối vào **cuối file**: mỗi lần đồng bộ upstream mà upstream cũng thêm mục ở cuối sẽ xung đột — giữ **cả hai** phía rồi chạy `cargo xtask assets --write`; + 1 mục `[[asset]]` cho `assets/licenses/LICENSE-cmap-resources.txt` (kind `document`, BSD-3-Clause, có sha256), cũng ở cuối file | Không (bắt buộc theo luật asset) |
| `ATTRIBUTION.md` | Sinh lại bằng `cargo xtask assets --write` | Đi cùng `ATTRIBUTION.toml` |
| `crates/ui-egui/src/theme.rs` | NX-UI (ROADMAP F2.2/F2.4): bảng màu Nexus PDF — app Rose `#F43F5E`, ink `#E11D48` (`APP_INK`, chữ trắng 4.7:1) và `#BE123C` (`APP_INK_DEEP`, chữ accent/hover 6.3:1); nền sáng slate `#F8FAFC`/`#F1F5F9`/`#E9EDF2`, viền `#CBD5E1`/`#E2E8F0`; tối `#0F172A`/`#1E293B`/`#020617`/`#273449`/`#334155`; bỏ xanh dương upstream `#1B63E0`; token mới `accent_hover`, `on_accent`, `info_bg`/`info_icon` (`#EFF6FF`/`#0369A1`, tối `#1E293B`/`#7DD3FC`, không theo accent); `Tokens: PartialEq`, `Tokens::for_context`; màu ngữ nghĩa `SELECTION` `#4F46E5`, `INFO`, `ERROR`, `WARNING`, `SUCCESS` thay bảng Spectrum của Adobe (`#1473E6`, `#D7373F`, `#E68619`, `#2D9D78`); màu chủ đạo người dùng chọn lưu trong bộ nhớ context egui (`set_accent`/`accent`, `with_accent` đẩy sáng/tối tới khi chữ đạt WCAG 4.5:1), `ACCENT_PRESETS` (Chàm, Tím, Bạc hà, Hổ phách, San hô, Da trời), `parse_hex`/`to_hex`, `contrast_ratio`; `apply` dùng `for_context`; test `nexus_palette_tests` | Có — bảng màu qua biến build |
| `crates/ui-egui/src/lib.rs` (thêm) | Trường `PdfCraftApp::accent: Option<Color32>`, `set_accent`; `persist` ghi `"accent": "#RRGGBB"` (không ghi khoá khi là màu app), `restore` đọc lỏng (rác = màu app); `set_option("accent", "#RRGGBB" | "none")` (dòng lệnh, `ui.set`, ví dụ `shot`); `logic()` áp màu qua `theme::set_accent` | Có — màu chủ đạo qua prefs |
| `crates/ui-egui/src/canvas.rs` | Thanh thông báo: `notice_colors` — thông báo thông tin (trường biểu mẫu, bảo mật, đã sửa file, bản chụp, chữ ký hợp lệ) dùng nền/icon trung tính `info_bg`/`info_icon` không theo màu chủ đạo; cảnh báo/lỗi (`triangle-alert`, `circle-x`) giữ nền accent nhạt; test `info_notice_bars_do_not_follow_the_accent` | Có — bảng màu |
| `crates/ui-egui/src/control.rs` | `ui.state` báo `accent` (`"#RRGGBB"` hoặc `null`) | Có |
| `crates/ui-egui/src/appearance.rs` | Preferences › Appearance: hàng màu chủ đạo (`nexus::accent_picker`) dưới hai thẻ chủ đề; hình thu nhỏ chủ đề vẽ bằng `Tokens::for_context` (có màu người dùng chọn) | Có |
| `crates/ui-egui/src/widgets.rs` (thêm) | Nút pill chính: hover `accent_hover`, chữ `on_accent` (upstream hover bằng `accent_text`, ở chế độ tối là màu sáng dưới chữ trắng); thẻ toast/tiến trình đảo màu dùng slate thay xám trung tính | Nên PR upstream (hover tối khó đọc) |
| `crates/ui-egui/src/edit_text_ui.rs` | Khung chọn chữ đang sửa: `theme::SELECTION` thay `#1473E6` (xanh Spectrum) | Có — bảng màu |
| `crates/ui-egui/src/content_ui.rs` | `SELECT_BLUE` → `theme::SELECTION`, `UNDRAWABLE` → `theme::ERROR` | Có — bảng màu |
| `crates/ui-egui/src/zoom_snap.rs` | Khung marquee zoom/snapshot: `theme::SELECTION` | Có — bảng màu |
| `crates/ui-egui/src/prepare.rs` | Khung chọn trường biểu mẫu: `theme::SELECTION` | Có — bảng màu |
| `crates/ui-egui/src/link_ui.rs` | Khung liên kết: `theme::SELECTION` | Có — bảng màu |
| `crates/ui-egui/src/object_ui.rs` | Khung đối tượng: `theme::SELECTION` | Có — bảng màu |
| `crates/ui-egui/src/camera_ui.rs` | Góc ảnh chụp camera: `theme::SELECTION` | Có — bảng màu |
| `crates/ui-egui/src/forms_ui.rs` | Viền trường đang focus: `theme::SELECTION` | Có — bảng màu |
| `crates/ui-egui/src/crop.rs` | Khung cắt trang: `theme::SELECTION` | Có — bảng màu |
| `crates/ui-egui/src/comments.rs` | Khung chọn ghi chú: `theme::SELECTION`; chữ nút Post `on_accent` | Có — bảng màu |
| `crates/ui-egui/src/comments_panel.rs` | Vạch thẻ ghi chú đang chọn: `t.accent` thay `#0165DD`; chữ nút Save `on_accent` | Có — bảng màu |
| `crates/ui-egui/src/sign_ui.rs` (thêm) | Khung vùng ký `theme::SELECTION`; trạng thái chữ ký `INFO`/`ERROR`/`SUCCESS`/`WARNING` thay bộ màu Spectrum | Có — bảng màu |
| `crates/ui-egui/src/combine_ui.rs` | `ERROR`/`WARNING` → `theme::ERROR`/`theme::WARNING`; liên kết "Unlock…" dùng `accent_text` (chữ) thay `accent` (nền) | Có — bảng màu |
| `crates/ui-egui/src/a11y_ui.rs` | `FAILED` → `theme::ERROR` | Có — bảng màu |
| `crates/ui-egui/src/home.rs` (thêm) | Icon file gần đây: `theme::APP_COLOR` (Rose) thay đỏ `#E03E3E` kiểu icon PDF của Acrobat | Có — bảng màu |
| `crates/ui-egui/src/folders_ui.rs` | Icon file trong thư mục: `theme::APP_COLOR` | Có — bảng màu |
| `crates/ui-egui/tests/ui.rs` (thêm) | Test vạch thả tab tìm pixel màu accent của theme thay "xanh dương" | Đi cùng `theme.rs` |
| `crates/ui-egui/tests/editing.rs` | So khung chọn với `theme::SELECTION` thay `#1473E6` | Đi cùng `theme.rs` |
| `crates/ui-egui/src/nexus.rs` (file mới, thêm) | `accent_picker`: hàng "Accent colour/Màu chủ đạo" (Màu ứng dụng, 6 màu gợi sẵn, chọn màu tuỳ ý) gọi `set_option("accent", …)` | — (file mới) |
| `crates/ui-egui/src/lib.rs` (NX-UI.4) | Bố cục kiểu WPS (ROADMAP NexusOffice F1.2/F1.9): trường `wps`, `embedded`, `host` (`nexus_wps::Host`), `wps_tab`, `wps_collapsed`, `backstage`; `eframe::App` gọi `frame_raw_input`/`frame_logic`/`frame_ui` công khai (cửa sổ chủ nhúng app không có `eframe::Frame`); khi `wps` vẽ hàng Tệp + dải lệnh thay thanh chế độ/rail/docking; khi `embedded` không đặt tiêu đề cửa sổ, không vẽ thanh tab tài liệu, không giữ nút đóng cửa sổ (`guard_quit`), không đổi cỡ giao diện; phông cài *cạnh* phông của trình soạn khác và chặn vẽ khi họ phông `medium`/`semibold` bị thay; Ctrl+N hỏi cửa sổ chủ; `set_option` `layout`, `embedded`, `wps-tab` | Có — bố cục là tuỳ chọn (`layout classic`) |
| `crates/ui-egui/src/editing.rs` | Lưu thành qua hộp thoại của cửa sổ chủ (chỉ PDF, `with_pdf_extension`) khi nhúng | Có — chỉ khi có `host.file_dialog` |
| `crates/engine/src/lib.rs` (NX-UI.6) | `Session::reserve_ids_from`: ứng dụng nhúng đánh số tài liệu trong dải riêng (`PdfCraftApp::embed`), hai tab không trùng `DocId` | Có |
| `crates/ui-egui/src/edit_text_ui.rs` | Tách `add_source_fonts` khỏi `install_source_fonts` và ghi pass cài phông, để khi nhúng cài lại phông nhúng của tài liệu đang sửa chữ cùng phông giao diện | Có |
| `crates/ui-egui/src/editing.rs` (NX-UI.6) | `has_unsaved_work` tính cả bình luận đang viết (ô soạn, trả lời, sửa thẻ, ô "Thêm bình luận") | Nên PR upstream |
| `crates/ui-egui/src/pickers.rs` | `ask_host`: chờ câu trả lời hộp thoại của cửa sổ chủ như một bộ chọn tệp gốc | Có |
| `crates/ui-egui/src/create_ui.rs` | `create.blank` khi nhúng: cửa sổ chủ mở PDF mới ở tab riêng | Có |
| `crates/ui-egui/src/theme.rs` (NX-UI.4) | `install_fonts_beside`, `fonts_installed` (nhúng chung cửa sổ với Nexus Docs/Sheet) | Có |
| `crates/ui-egui/src/i18n/vi.tsv` | "Save changes?" → "Lưu thay đổi?" như Nexus Docs/Sheet (nhãn riêng của bố cục WPS nằm trong bảng `NEXUS_VI` của `nexus_wps.rs`, không thêm khoá vào catalog: test upstream đòi mọi khoá có ở cả 22 ngôn ngữ) | Không (bản dịch của fork) |
| `crates/engine/src/lib.rs` (NX-UI.7) | `reserve_ids_from` trả `Result` và từ chối dải quá nửa `u64`; `commit_open` cộng bão hoà (`next_id` không tràn); `Session::open_unsaved(name, path, bytes)`: tài liệu chưa lưu giữ tệp gốc (`open_new` gọi nó) | Có |
| `crates/ui-egui/src/nexus_wps.rs` (NX-UI.7) | Tài liệu giao cho tab khác (`AdoptedDocument`) mang theo tệp gốc và khoá bản khôi phục (bản khôi phục không bị xoá khi giao; `open_adopted` ghi lại ngay dưới cùng khoá); tài liệu không giao được thì ở lại, báo một lần | Có |
| `crates/ui-egui/src/editing.rs` (NX-UI.7) | Thẻ bình luận đang sửa chỉ tính là chưa lưu khi chữ khác nội dung cũ | Nên PR upstream |
| `crates/ui-egui/src/nexus_wps.rs` (`restyle`), `edit_text_ui.rs` (NX-UI.7) | Khung bỏ qua khi cài lại phông (nhúng) giữ bàn phím cho ô sửa chữ (`LineEditor::keep_focus`), không áp dụng sửa đổi sau lưng người dùng | Có |
| `crates/ui-egui/src/nexus_wps.rs` (NX-UI.8) | Tài liệu phải nhập mật khẩu mới mở được (tài liệu khôi phục đã mã hoá) thì giữ lại ở tab nguồn, không giao đi rồi biến mất (`stray_bytes` thử mở lại); tài liệu giữ lại có thanh "Cũng mở trong tab này" để chuyển qua lại, lưu hoặc đóng; thông báo "Không chuyển được … sang tab riêng"; tab ẩn vẫn chờ tài liệu đang mở và giao đi (`background_frame`), `handing_over` | Có |
| `crates/ui-egui/src/editing.rs` (NX-UI.8) | Lưu (cả nút Lưu trong hộp "Lưu thay đổi?" khi đóng) áp dụng đoạn đang mở trong ô sửa chữ trước (`apply_open_text_edit`) | Nên PR upstream |
| `crates/ui-egui/src/editing.rs` (NX-UI.9) | Đoạn đang sửa không áp dụng được thì giữ nguyên ô sửa chữ với bản nháp, dừng Lưu và báo (trước đó bản nháp mất mà vẫn lưu) | Nên PR upstream |
| `crates/ui-egui/src/nexus_wps.rs` (NX-UI.9) | `stray_bytes` chỉ mở lại để thử khi tài liệu có mã hoá (bỏ một lần phân tích cả tệp trên luồng giao diện); lý do không giao được nói bằng ngôn ngữ giao diện, lý do tiếng Anh của engine ghi vào log | Có |
| `crates/ui-egui/src/nexus_wps.rs`, `lib.rs` (NX-UI.9) | Tài liệu giao cho host ở lại tab nguồn (ẩn, `nexus_handed`) tới khi host trả lời `adoption_answered`: mở được thì đóng ở đây, không mở được thì giữ lại (`nexus_kept`) và báo; `AdoptedDocument::source` | Có |
| `crates/ui-egui/src/editing.rs` (NX-UI.9) | Lưu dừng vì đoạn đang sửa không áp dụng được thì ghi lý do của engine vào log | Nên PR upstream |
| `apps/pdfcraft/src/main.rs` (NX-UI.5) | Nexus PDF mở ở bố cục WPS (`--layout classic` để về bố cục upstream) | Có |
| `apps/pdfcraft-web/src/main.rs` (NX-UI.5) | Bản web cũng dùng bố cục WPS | Có |

### GUID WiX mới (Nexus PDF riêng, không dùng chung với PdfCraft hay app Nexus khác)

| Mục | GUID cũ (upstream PdfCraft) | GUID mới (Nexus PDF) |
|---|---|---|
| UpgradeCode (cả Upgrade Id OnlyDetect) | `EA00C367-7270-4C50-9D8B-A7A5C88AD27F` | `CA922A7F-167E-414E-86E2-875DF949BBFC` |
| Component `PdfcraftApp` | `E724C1C6-091F-4E21-A685-80D089FDBE21` | `D114091A-AEAF-481B-B7B9-638670FF14E1` |
| Component `PdfcraftStartMenuShortcut` | `DC028192-C233-4F99-9A77-ED458C60DF26` | `24AF82A2-A27C-4817-95DF-C779227B63A8` |
| Component `PdfcraftDesktopShortcut` | `F6948627-32AB-4D78-8865-0FDF417CE442` | `0F36E294-1953-45F5-A36C-364A78A9FAB7` |
| Component `PdfcraftPdfContextMenu` | `439F084F-657D-4AC2-A4CC-F099C518D71F` | `794DEC2C-BC7B-4D7B-8DAA-3D840BC8BD81` |
| Component `PdfcraftImageContextMenu` | `1B783BF3-12DF-4588-AF80-E2CEA8BC1E5A` | `F289E511-669D-4966-B3E3-77A920CAC254` |
| Component `PdfcraftCli`, licence/NOTICE components, models | (tự sinh theo đường dẫn) | tự sinh — thư mục cài `Nexus PDF` và tên file đã khác |
| Component `PdfcraftAttribution`, ComponentGroup `PdfcraftFontLicences` (mỗi file trong `licenses\` một component) | (không có ở upstream) | tự sinh — `ATTRIBUTION.md` và OFL của font nhúng |

Sinh bằng `python3 -c "import uuid; print(str(uuid.uuid4()).upper())"` ngày 2026-10-17 (container không
có `uuidgen`). Khác upstream và khác wordcraft (`B8F9CA5A…`), gridcraft (`AD43085E…`), deckcraft (`96D068F9…`).

### File mới (không tính là patch)
`crates/engine/src/nexus.rs`, `crates/ui-egui/src/i18n/brand.rs`, `crates/ui-egui/src/nexus.rs`,
`crates/ui-egui/src/nexus_wps.rs` (bố cục WPS, móc cho cửa sổ chủ, test kittest), `crates/ui-egui/src/nexus_backstage.rs` (trang Tệp sáng), `crates/ui-egui/tests/nexus_embed.rs` (nhúng: phông của trình sửa chữ, hai ứng dụng một cửa sổ, bình luận dở),
`crates/automation/tests/nexus_brand.rs` (Producer, tác giả mặc định, `/Prop_Build` chữ ký, nhãn lệnh,
mô tả tool, `serverInfo`), `apps/pdfcraft-cli/tests/nexus_brand.rs` (USAGE/`--version`/tiền tố lỗi/lỗi
file rác), `packaging/linux/vn.sonnytech.nexus.pdf.desktop`, `packaging/linux/vn.sonnytech.nexus.pdf.metainfo.xml.in`
(MIME dùng lại file upstream `ai.storyteller.pdfcraft.mime.xml`, cài dưới tên APP_ID), `nexus/assets/*`
(logo tạm `nexus-pdf.svg` + PNG/ICO/ICNS/hicolor sinh bằng `nexus/assets/render-icons.py`, `LICENSE-CC0.txt`).
`assets/licenses/LICENSE-cmap-resources.txt` (vòng sửa 2): giấy phép BSD-3-Clause của tài nguyên CMap Adobe mà hayro-cmap 0.1.0 biên dịch vào (`assets/cmaps.brotli`), chép nguyên văn từ `assets/LICENSE.txt` của crate (checksum khớp Cargo.lock) kèm đoạn đầu mô tả; đóng gói thành `LICENSE-cmap-resources.txt` ở mọi gói.

## File khác

| File | Lý do | Thay được bằng white-label? |
|---|---|---|
| `deny.toml` | Bỏ qua RUSTSEC-2026-0370/0371/0372 (`zvariant`, `zbus_names`, `zvariant_derive` ngừng bảo trì vì gộp vào zbus 6, không phải lỗ hổng; đến qua eframe → accesskit_unix, chưa có bản nâng cấp). Cảnh báo ra ngày 2026-10-17 làm đỏ cổng `cargo deny` cả trên `nexus`; bỏ dòng này khi upstream xử lý | Không |

## Rà soát gọi mạng và chuỗi ghi vào file (guard §5)
- Không telemetry/crash report. Kiểm tra cập nhật chỉ khi người dùng bấm Help ▸ Check for updates →
  `api.github.com/repos/nhdai111983/pdfcraft/releases/latest` (fork), không còn gọi storytold.
- Link app tự mở: chỉ repo fork (Help ▸ Nexus PDF on GitHub, About, Home). Discord/getartcraft đã bỏ
  khỏi app, CLI, MCP, MSI (ARP), macOS Credits, metainfo, README.
- Chuỗi ghi vào file người dùng: Producer PDF mới, tác giả mặc định ghi chú (MCP/run), `/Prop_Build`
  của chữ ký và dấu thời gian, mô tả ICC sRGB (PDF/A), họ phông Type 3 chữ trộn script, `<title>`
  trang in web. Không đổi: dấu `%PdfCraft` trong luồng nội dung (khoá định dạng, vô hình).
- Dependency git `storytold/craft-libs` chỉ là nguồn build (mirror T5/§4.8).

## Việc còn nợ (đã allowlist có lý do, hoặc không bị brand-check bắt)
- Đề xuất PR upstream (white-label): `APP_NAME`/`SETTINGS_DIR`/`APP_ID`/URL lấy từ biến build
  (`PDFCRAFT_APP_NAME`…) và placeholder `{app}` trong catalog → bỏ phần lớn patch ở trên.
- `docs/brand/` (logo ArtCraft) vẫn nằm trong repo (không còn được biên dịch hay đóng gói); xoá khỏi
  fork cần sửa `ATTRIBUTION.toml`/`xtask assets` — để Henry quyết.
- Icon tài liệu PDF Windows (`assets/app-icon/pdfcraft-document.ico`, MIT/Apache, không có chữ PdfCraft)
  vẫn của upstream.
- Log: dòng info Wayland trong `apps/pdfcraft/src/decorations.rs` vẫn nói PdfCraft (chỉ trong file log).
- **NỢ — gói FreeBSD và Flatpak** (ngoài phạm vi T12): `packaging/freebsd/package.sh` và
  `packaging/linux/flatpak*` (manifest `ai.storyteller.pdfcraft.bundle.yml`, `flatpak-bundle.sh`) chưa đổi:
  vẫn app id/tên PdfCraft và model ở `share/pdfcraft/models` (app vẫn tìm được vì `Models::dirs_beside_exe`
  giữ đường dẫn upstream). Chưa phát hành (workflow freebsd tắt, Flatpak không nằm trong release Nexus).
  Cần đổi app id, tên, icon và thư mục model trước khi bật.
- `packaging/icons.sh` và file `packaging/linux/ai.storyteller.pdfcraft.*` (MIME rỗng, cài dưới tên
  `vn.sonnytech.nexus.pdf.xml`, nên không đụng file của gói PdfCraft) chưa đổi.
- DMG macOS: layout/nền gắn tên PdfCraft → DMG Nexus mở với layout mặc định; sinh lại trên macOS (Task 12).
- Công cụ dev `cargo xtask` (screenshots, demo-pdf, macos-integration) và `parity/` vẫn nói PdfCraft.
