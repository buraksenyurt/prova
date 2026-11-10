# Prova

Öğrencilerin sınava hazırlanırken testleri **tamamen tarayıcıda** çözebildiği, Rust + [Leptos](https://leptos.dev) (CSR) ile yazılmış bir WebAssembly uygulaması. Test dosyası ve cevaplar hiçbir sunucuya gönderilmez.

**Canlı:** https://buraksenyurt.github.io/prova/

## Çalıştırma

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
trunk serve --open

# release modda
trunk serve --release --open
```

Testler (tarayıcı gerekmez):

```bash
cargo test
```

## Özellikler

- **Ders sınavları:** "Yapay Zeka Destekli Yazılım Geliştirme" dersinin sınavları uygulamayla birlikte gelir ve ana sayfadan tek tıkla açılır:
  - Final örnek soruları, 27 soru: `?exam=ai-dev-final-00`
  - Senaryo Soruları I, 10 açıklamalı senaryo sorusu (RAG, MCP, Skill, döngü mühendisliği, LoRA, güvenlik): `?exam=ai-dev-practice-01`
- **Yükleme:** JSON test dosyasını veya [Markdown ile yazılmış bir sınavı](#markdown-ile-sınav-yazmak) sürükle-bırak veya seç, bir adresten yükle ya da gömülü örnek testi dene.
- **Paylaşılabilir link:** `https://buraksenyurt.github.io/prova/?quiz=<adres>` testi açılışta otomatik yükler. Raw GitHub veya gist adresleri CORS'a izin verdiği için doğrudan çalışır; GitHub'daki `.../blob/...` sayfa linki de otomatik olarak raw adrese çevrilir.
- **İki mod**
  - *Antrenman:* her cevaptan sonra doğru şık ve açıklama gösterilir, süre baskısı yoktur.
  - *Sınav simülasyonu:* geri sayım, cevap değiştirme, "sonra bak" işareti; sonuçlar en sonda. Süre dolunca test kendiliğinden biter.
- **Soru ve şık karıştırma.** Karıştırma, tekrar üretilebilir olsun diye tohumlu (seeded) bir PRNG ile yapılır. `lock_options` işaretli sorularda ("Hepsi", "Yalnızca I ve II" gibi) şıkların sırası korunur.
- **Şık eleme:** sağ tık, uzun basma veya ⊘ düğmesiyle şıkkın üstü çizilir.
- **Olumsuz kök vurgusu:** soru kökündeki "değildir", "yanlıştır", "söylenemez", "NOT" gibi kelimeler otomatik olarak işaretlenir.
- **Sonuç ekranı:** skor, doğru/yanlış/boş sayıları, süre, zayıftan güçlüye konu kırılımı, filtrelenebilir gözden geçirme listesi ve **yanlış ve boşları tekrar çöz** düğmesi.
- **Kaldığı yerden devam:** deneme her değişiklikte `localStorage`'a yazılır, sayfa yenilense de kaybolmaz. Her testin son 10 denemesi saklanır.
- **Konulara göre gelişim:** giriş ekranında her konunun son 5 tam denemedeki başarısı küçük grafiklerle, sonuç ekranında ise önceki denemeye göre değişim (▲/▼) gösterilir. "Tekrar çöz" denemeleri bu karşılaştırmaya katılmaz.
- **Kod renklendirme ve formüller:** kod blokları highlight.js, `$...$` / `$$...$$` formüller KaTeX ile gösterilir. Kütüphaneler repoda (`assets/vendor`) durur ve yalnızca içerikte kod ya da formül varsa yüklenir; CDN'e bağımlılık yoktur.
- **Klavye:** `A–H` / `1–9` şık seçer, `←` / `→` gezinir, `F` işaretler.
- TR/EN arayüz, açık/koyu tema, mobil uyumlu yerleşim.

## Test dosyası formatı (`schema_version: 1`)

Örnek: [`samples/demo.json`](samples/demo.json)

```json
{
  "schema_version": 1,
  "id": "ai-dev-exam-00",
  "title": "Final Sınavı Örnek Soruları",
  "description": "Markdown (isteğe bağlı)",
  "assets_base_url": "https://raw.githubusercontent.com/<kullanıcı>/<repo>/main/",
  "settings": { "shuffle_questions": true, "shuffle_options": true, "time_limit_minutes": 60 },
  "questions": [
    {
      "id": "q01",
      "topics": ["Test", "TDD"],
      "stem": "Markdown",
      "options": [
        { "id": "a", "text": "Markdown" },
        { "id": "b", "text": "Markdown" }
      ],
      "lock_options": false,
      "answer": "a",
      "explanation": "Markdown (isteğe bağlı)"
    }
  ]
}
```

| Alan | Açıklama |
|---|---|
| `id` | Testin kimliği. Deneme geçmişi bu kimlikle saklanır. |
| `assets_base_url` | Markdown içindeki göreli görsel yolları (`./images/x.png`) bu adrese göre çözülür. |
| `settings` | Yazarın önerdiği varsayılanlar; öğrenci başlamadan önce değiştirebilir. Süre yalnızca sınav modunda uygulanır. |
| `options[].id` | Şıkkın sabit kimliği. Ekrandaki A, B, C harfleri gösterim sırasına göre verilir; karıştırma bu sayede mümkün. |
| `lock_options` | Şıkların sırası anlam taşıyorsa `true`. |
| `answer` | Doğru şıkkın `id`'si. Ya tüm sorularda olmalı ya da hiçbirinde. |
| `topics` | Sonuç ekranındaki konu kırılımı için. |

Metin alanlarında Markdown kullanılabilir: kod blokları, tablolar, listeler, görseller, `$...$` ve `$$...$$` formüller. İçerikteki ham HTML çalıştırılmaz, metin olarak gösterilir. Bilinmeyen alanlar yok sayılır.

Dosyadaki hatalar soru kimliğiyle birlikte anlaşılır bir mesajla gösterilir (geçersiz JSON'da satır/sütun bilgisiyle).

## Ders sınavları

Uygulamayla birlikte gelen sınavlar [`exams/`](exams) klasöründe durur ve [`exams/catalog.json`](exams/catalog.json) ile listelenir. Trunk bu klasörü olduğu gibi derlemeye kopyalar; uygulama kataloğu açılışta okur.

Yeni bir sınav eklemek için:

1. Sınavı [Markdown biçiminde](#markdown-ile-sınav-yazmak) (ya da JSON olarak) `exams/` altına koy. Görseller `exams/images/` altına, Markdown'da `./images/...` yoluyla.
2. Dosyaya `<!-- id: ... -->` ile bir kimlik ver ve `catalog.json`'a aynı `id` ile bir kayıt ekle:

```json
{
  "id": "ai-dev-final-00",
  "title": "Final Sınavı Örnek Soruları",
  "course": "Yapay Zeka Destekli Yazılım Geliştirme",
  "description": "Kısa açıklama",
  "file": "ai-dev-final-00.md",
  "questions": 27
}
```

3. `cargo test` çalıştır. `shipped_exams_are_valid` testi her sınavın uyarısız dönüştüğünü, tüm cevapların olduğunu, kimliğin ve soru sayısının katalogla aynı olduğunu ve göreli görsellerin klasörde bulunduğunu doğrular. Aynı test CI'da her push'ta çalışır.

`ai-dev-final-00.md`, ders reposundaki [Exam00.md](https://github.com/buraksenyurt/ai-based-development-lecture/blob/main/Exam00.md) dosyasının kopyasıdır; konu ilerlemesi için her soruya `<!-- topics: ... -->` etiketi eklenmiştir.

## Markdown ile sınav yazmak

Sınavları JSON yerine doğrudan Markdown ile yazabilirsin. `.md` dosyası uygulamaya yüklendiğinde tarayıcıda JSON'a çevrilir; giriş ekranındaki **JSON olarak indir** düğmesiyle dönüştürülmüş hali alınabilir. Örnek: [`samples/exam-sample.md`](samples/exam-sample.md)

````markdown
# Sınav başlığı
İsteğe bağlı açıklama.

<!-- id: final-00 -->
<!-- time: 40 -->

## Soru 1
Soru kökü: kod blokları, tablolar, görseller, "- I." öncülleri olabilir.

- A) Birinci şık
- B) İkinci şık

> Açıklama: İsteğe bağlı; şıklardan sonra '>' ile.

<!-- topics: RAG, Test -->

## Cevap Anahtarı
| Soru No | Doğru Cevap |
|---|---|
| 1 | B |
````

- Sorular `## Soru N` veya `## Question N` başlıklarıyla başlar. Şıklar `- A) ...` (ya da `- **A)** ...`) biçimindedir ve A'dan başlayıp sırayla gider.
- Cevaplar `## Cevap Anahtarı` / `## Answer Key` tablosundan ya da soru içindeki `<!-- answer: B -->` yorumundan okunur. Ya tüm soruların cevabı olmalı ya da hiçbirinin.
- `<!-- topics: ... -->` konu etiketleri, `<!-- lock -->` şık sırasını korur. "Hepsi", "Yalnızca I ve II" gibi şıklarda sıra zaten otomatik korunur.
- Açıklama şıklara harfle atıf yapıyorsa (`**A**` yanlış, çünkü ...) soruya `<!-- lock -->` ekle; aksi halde şıklar karıştırıldığında harfler tutmaz. Dönüştürücü bu durumda uyarı verir.
- Dosya bir URL'den yüklendiğinde göreli görsel yolları (`./images/x.png`) dosyanın klasörüne göre çözülür.

Komut satırından:

```bash
cargo run --bin md2quiz -- Exam00.md -o exam00.json \
    --assets-base-url https://raw.githubusercontent.com/<kullanıcı>/<repo>/main/
```

`md2quiz --help` diğer seçenekleri (`--id`, `--time`, `--shuffle-questions`, `--no-shuffle-options`) listeler. Çıktı yazılmadan önce uygulamanın kendi şema doğrulamasından geçirilir.

## Mimari

```
src/
├── domain/       Saf model, tarayıcı API'si yok → cargo test ile test edilir
│   ├── quiz.rs       Quiz, Question, Choice; AnswerKey ayrı tutulur
│   ├── attempt.rs    Deneme durumu: sıra, cevaplar, işaretler, süre, kurallar
│   ├── summary.rs    Skor ve konu kırılımı
│   ├── history.rs    Deneme geçmişi ve konu bazlı gelişim
│   └── rng.rs        Tohumlu karıştırma
├── dto/          Dosya formatları (v1) → domain dönüşümü ve doğrulama
├── convert.rs    Sınav Markdown'ı → JSON (uygulama ve md2quiz ortak kullanır)
├── bin/md2quiz.rs  Komut satırı dönüştürücü
├── source/       QuizSource trait'i: dosya, URL, katalog, gömülü/saklanan metin
├── grading/      Grader trait'i: LocalGrader (cevap anahtarıyla, tarayıcıda)
├── markdown.rs   Güvenli Markdown render, görsel yolları, olumsuz kök vurgusu
exams/            Uygulamayla gelen sınavlar ve catalog.json
assets/
├── enhance.js    highlight.js ve KaTeX'i ihtiyaç anında yükleyip uygular
└── vendor/       highlight.js 11.11.1 ve KaTeX 0.16.25 (lisanslarıyla)
├── storage.rs    localStorage: devam eden deneme, son test, geçmiş
├── i18n.rs       TR/EN, eksik çeviri derleme hatası verir
├── app.rs        Uygulama durumu ve ekranlar arası akış
└── components/   home, intro, runner, results, question, header
```

### Servise geçişe hazırlık

İki soyutlama, ileride soruların ve değerlendirmenin bir servise taşınmasını arayüze dokunmadan mümkün kılar:

- **`QuizSource`**: soruların nereden geldiği. Bir `ApiSource` eklemek yeterli.
- **`Grader`**: cevabın doğru olup olmadığına kim karar verir. Sorular (`Question`) doğru cevabı **taşımaz**; cevap anahtarı yalnızca `LocalGrader`'ın içindedir. `RemoteGrader` aynı toplu (batch) API'yi kullanarak cevapları servise gönderebilir. Antrenman modu tek soru, sınav modu tüm soruları tek çağrıda yollar.

`answer` alanı olmayan bir dosya bugün de geçerlidir. Uygulama onu yükler ama değerlendirici olmadığı için başlatmaya izin vermez.

> **Not:** Şimdilik cevaplar JSON dosyasında açıkça durur. Uygulama antrenman amaçlı olduğu için bu bilinçli bir karardır; cevapları gizlemeye çalışmak yerine Faz 3'te değerlendirme servise taşınacak.

## Yol haritası

- ~~**Faz 2:** kod renklendirme, KaTeX ile formüller, Exam Markdown → JSON dönüştürücü, konu bazlı ilerleme geçmişi.~~
- **Faz 3 (beklemede):** `ApiSource` + `RemoteGrader`, servis tarafı, öğrenci kimliği ve sonuçların toplanması. Statik soru setleri ve cevaplar ayrı bir repoya taşınabilir; o zamana kadar sınavlar `exams/` klasöründe uygulamayla birlikte yayınlanıyor. Katalog adresi (`CATALOG_URL`) servise yönlendirilerek bu yapı kademeli olarak taşınabilir.

## Yayınlama

`main` dalına yapılan her push'ta GitHub Actions testleri çalıştırır ve uygulamayı GitHub Pages'e yayınlar ([`.github/workflows/deploy.yml`](.github/workflows/deploy.yml)). İlk seferde repo ayarlarından **Settings → Pages → Source: GitHub Actions** seçilmelidir.

### Sürüm çıkarma

`v*` biçiminde bir tag push'landığında [`.github/workflows/release.yml`](.github/workflows/release.yml) testleri çalıştırır, uygulamayı derler, `dist` klasörünü `prova-<tag>.zip` olarak paketler ve sürüm notları commit'lerden üretilen bir GitHub Release oluşturur. Paket göreli adreslerle derlendiği için herhangi bir statik sunucuda, alt dizinde de çalışır.

Tag, `Cargo.toml`'daki sürümle aynı olmalıdır; değilse workflow durur.

```bash
# Cargo.toml: version = "0.1.0"
git tag -s v0.1.0 -m "Faz 1"
git push origin v0.1.0
```
