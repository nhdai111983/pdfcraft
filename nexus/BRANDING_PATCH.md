# BRANDING_PATCH

Danh sách file mà nhánh `nexus` sửa so với upstream vì lý do thương hiệu.
Mỗi mục là đường dẫn file chính xác trong dấu backtick (không dùng glob/thư mục).

Giá trị: sản phẩm **Nexus PDF**, bộ **NexusOffices**, nhà phát hành **Sonnytech**, app id
`vn.sonnytech.nexus.pdf`, tài sản phát hành `NexusPDF-*`, binary cài đặt `nexus-pdf`/`nexus-pdf-cli`
(MSI, zip portable, deb/rpm/AppImage/tar, zip CLI macOS; tên build Cargo vẫn là `pdfcraft`/`pdfcraft-cli`,
bundle macOS `Nexus PDF.app` với executable `NexusPDF`). Thư mục người dùng: `NexusPDF` (macOS/Windows),
`nexuspdf` (XDG), `NexusPDFData` (portable) — một nguồn duy nhất `crates/engine/src/nexus.rs`. Không có
khoảng trắng và dạng XDG là chữ thường của nó, để `eframe::storage_dir` (macOS giữ tên nhưng đổi khoảng
trắng, Linux bỏ khoảng trắng và viết thường) và các đường dẫn tự dựng (khôi phục, clipboard, Digital IDs,
socket một phiên bản) cùng ra **một** thư mục trên mọi nền tảng, như `PdfCraft`/`pdfcraft` của upstream
(review T10 vòng 1; test `settings_dir_survives_eframe_normalisation`).

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
  `PdfCraft → Nexus PDF`, `ArtCraft → NexusOffices`. Áp **một lần** trong đường tra chuỗi:
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
| `crates/ui-egui/src/dialogs.rs` | About: logo Nexus vẽ bằng code + tên `APP_NAME`; thay "Part of" + wordmark ArtCraft bằng `nexus::about_lines` (bộ NexusOffices/Sonnytech, câu ghi công, bản quyền upstream) | Có |
| `crates/ui-egui/src/home.rs` | Thẻ "Join the ArtCraft community"/Discord → `nexus::home_card` (logo, NexusOffices, link mã nguồn) | Có |
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
| `apps/pdfcraft/build.rs` | Resource Windows: icon Nexus, ProductName/FileDescription Nexus PDF, CompanyName Sonnytech, LegalCopyright (ghi công), OriginalFilename `nexus-pdf.exe` | Có |
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
| `packaging/windows/pdfcraft.wxs` | MSI: Name/Manufacturer **Sonnytech**, **UpgradeCode + mọi GUID component mới**, `Program Files\Nexus PDF`, `nexus-pdf.exe`/`nexus-pdf-cli.exe`, ProgId `Sonnytech.NexusPDF.Document`, `Applications\nexus-pdf.exe`, App Paths, `Software\Sonnytech\Nexus PDF\…`, `RegisteredApplications\NexusPDF`, menu chuột phải `NexusPDF.Edit`/`NexusPDF.CreatePdf` ("Edit with Nexus PDF"), shortcut `Nexus PDF.lnk`; ARP: bỏ getartcraft/Discord, About/Update → repo fork; NexusOffices: component `PdfcraftAttribution` (`ATTRIBUTION.md`, GUID tự sinh), thư mục `licenses\` + ComponentGroup `PdfcraftFontLicences` (`<Files Include="$(var.BinDir)\licenses\*">`, có điều kiện `FontLicences`); vòng sửa 2: chú thích — nhóm `FontLicences` phủ mọi văn bản giấy phép bên thứ ba (OFL, Lucide, Foxit, CMap Adobe) | Một phần (biến WiX) |
| `packaging/windows/installer-ui.wxs` | Nhãn checkbox "Edit with Nexus PDF" (các hộp thoại khác dùng `[ProductName]`) | Một phần |
| `packaging/windows/package.ps1` | `NexusPDF-*.msi/.zip`, exe đổi tên `nexus-pdf(-cli).exe`, icon Nexus; chép `ATTRIBUTION.md` vào thư mục stage (MSI) và zip portable; vòng sửa 1: dựng `licenses\` = OFL craft-fonts (`OFL-<family>.txt`) + `assets\fonts\OFL-*.txt` (tên duy nhất không phân biệt hoa thường; ném lỗi khi `CRAFT_FONTS_REQUIRED=1` mà không có OFL), truyền `-d FontLicences=yes|no` cho WiX, zip portable lấy OFL từ `licenses\`; vòng sửa 2: `Add-Licence` thêm `LICENSE-lucide.txt`, `LICENSE-FOXIT.txt`, `LICENSE-cmap-resources.txt` vào `licenses\` (bắt buộc); `FontLicences` nay phủ mọi văn bản giấy phép bên thứ ba (luôn `yes` với Nexus PDF) | Một phần |
| `packaging/windows/test-msi.ps1` | Smoke test MSI theo Manufacturer/khoá/nhãn mới; khẳng định `NOTICE`/`ATTRIBUTION.md` nằm trong `INSTALLFOLDER` và mọi `assets\fonts\OFL-*.txt` có trong `licenses\`; vòng sửa 2: so tên dài (`SHORT|long` → phần sau `|`) cho `INSTALLFOLDER` và `licenses\`, khẳng định thêm `LICENSE-lucide.txt`, `LICENSE-FOXIT.txt`, `LICENSE-cmap-resources.txt` | Đi cùng `pdfcraft.wxs` |
| `packaging/windows/sign.ps1` | Mô tả chữ ký `/d Nexus PDF`, `/du` repo fork | Một phần |
| `packaging/windows/portable.txt` | Hướng dẫn chế độ portable (nằm trong zip) | Không |
| `packaging/windows/security-audit.ps1` | Ví dụ tên file trong trợ giúp script | Không |
| `packaging/macos/Info.plist.in` | `Nexus PDF`, executable/icon `NexusPDF`, bundle id `vn.sonnytech.nexus.pdf`, copyright (ghi công) | Một phần |
| `packaging/macos/Credits.rtf` | About macOS: NexusOffices, ghi công, link fork (bỏ getartcraft/Discord) | Không |
| `packaging/macos/package.sh` | `Nexus PDF.app`, `NexusPDF-*.dmg`, `NexusPDF-cli-*.zip` chứa `nexus-pdf-cli`, icns Nexus, tên volume; không chép `dmg-layout.DS_Store`/nền (gắn tên PdfCraft); `copy_notices` chép `LICENSE-MIT`/`LICENSE-APACHE`/`NOTICE`/`ATTRIBUTION.md` và OFL của font repo vào `Contents/Resources` của app (trước khi ký) | Một phần |
| `packaging/linux/package.sh` | `APP_ID=vn.sonnytech.nexus.pdf`, `/usr/bin/nexus-pdf(-cli)`, `NexusPDF-*`, icon `nexus/assets/hicolor`, maintainer Sonnytech, zsync trỏ fork; model OCR ở `usr/share/nexus-pdf/models` (không đụng `/usr/share/pdfcraft` của gói PdfCraft đã cài); AppImage giữ `usr/share/doc/nexus-pdf` (giấy phép, `NOTICE`, `ATTRIBUTION.md`, OFL) thay vì bỏ ra ngoài | Một phần |
| `packaging/linux/nfpm.yaml` | Gói `nexus-pdf`, vendor Sonnytech, homepage fork, binary, ghi công | Một phần |
| `packaging/env.sh` | `copy_docs` = README/LICENSE/COPYRIGHT tuỳ có + OFL craft-fonts + `copy_notices` mới **bắt buộc** (`LICENSE-MIT`/`LICENSE-APACHE`/`NOTICE` — Apache-2.0 §4(d) — /`ATTRIBUTION.md`, README Nexus trỏ tới) + `assets/fonts/OFL-*.txt` (Inter, JetBrains Mono, Dancing Script nhúng trong binary; báo lỗi khi trùng tên); mọi `cp` có `|| return 1`; `copy_font_licences` báo lỗi khi `CRAFT_FONTS_REQUIRED=1` mà không có OFL. Dùng cho deb/AppImage/tar, zip CLI, zip web (và FreeBSD của upstream); vòng sửa 2: `copy_notices` chép thêm (bắt buộc) `assets/icons/LICENSE-lucide.txt` → `LICENSE-lucide.txt` (ISC + MIT Feather), `vendor/hayro-interpret/assets/LICENSE_FOXIT` → `LICENSE-FOXIT.txt` (font Foxit BSD-3), `assets/licenses/LICENSE-cmap-resources.txt` (CMap Adobe BSD-3) vào thư mục doc/`Contents/Resources`; cập nhật chú thích `copy_font_licences` | Nên PR upstream (gói upstream thiếu ATTRIBUTION.md/OFL của font repo) |
| `packaging/web/package.sh` | Zip `NexusPDF-web-<ver>.zip` | Một phần |
| `packaging/web/README.md` | Được đóng gói thành `HOSTING.md` trong zip web | Không |
| `README.md` | README Nexus PDF ngắn (vi trước, có English), câu ghi công "Nexus PDF được phát triển dựa trên PdfCraft của ArtCraft (MIT/Apache-2.0)" chỉ bằng chữ (không logo/Discord/getartcraft); `copy_docs` đóng gói nó | Không (README của fork) |
| `Cargo.toml` | `[workspace.package] repository` → `https://github.com/nhdai111983/pdfcraft` (brand-check `brand-url`; metadata, không code nào đọc `CARGO_PKG_REPOSITORY`) | Có — metadata của fork |
| `ATTRIBUTION.toml` | 14 mục `[[asset]]` cho icon Nexus (`nexus/assets/**`, Sonnytech, CC0-1.0, có sha256), nối vào **cuối file**: mỗi lần đồng bộ upstream mà upstream cũng thêm mục ở cuối sẽ xung đột — giữ **cả hai** phía rồi chạy `cargo xtask assets --write`; + 1 mục `[[asset]]` cho `assets/licenses/LICENSE-cmap-resources.txt` (kind `document`, BSD-3-Clause, có sha256), cũng ở cuối file | Không (bắt buộc theo luật asset) |
| `ATTRIBUTION.md` | Sinh lại bằng `cargo xtask assets --write` | Đi cùng `ATTRIBUTION.toml` |

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
`crates/automation/tests/nexus_brand.rs` (Producer, tác giả mặc định, `/Prop_Build` chữ ký, nhãn lệnh,
mô tả tool, `serverInfo`), `apps/pdfcraft-cli/tests/nexus_brand.rs` (USAGE/`--version`/tiền tố lỗi/lỗi
file rác), `packaging/linux/vn.sonnytech.nexus.pdf.desktop`, `packaging/linux/vn.sonnytech.nexus.pdf.metainfo.xml.in`
(MIME dùng lại file upstream `ai.storyteller.pdfcraft.mime.xml`, cài dưới tên APP_ID), `nexus/assets/*`
(logo tạm `nexus-pdf.svg` + PNG/ICO/ICNS/hicolor sinh bằng `nexus/assets/render-icons.py`, `LICENSE-CC0.txt`).
`assets/licenses/LICENSE-cmap-resources.txt` (vòng sửa 2): giấy phép BSD-3-Clause của tài nguyên CMap Adobe mà hayro-cmap 0.1.0 biên dịch vào (`assets/cmaps.brotli`), chép nguyên văn từ `assets/LICENSE.txt` của crate (checksum khớp Cargo.lock) kèm đoạn đầu mô tả; đóng gói thành `LICENSE-cmap-resources.txt` ở mọi gói.

## File khác

| File | Lý do | Thay được bằng white-label? |
|---|---|---|

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
