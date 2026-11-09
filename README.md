# Prova

Öğrencilerin sınava hazırlanırken testleri **tamamen tarayıcıda** çözebildiği, Rust + [Leptos](https://leptos.dev) (CSR) ile yazılmış bir WebAssembly uygulaması. Test dosyası ve cevaplar hiçbir sunucuya gönderilmez.

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

## Özellikler (Faz 1)

- **Yükleme:** JSON dosyasını sürükle-bırak veya seç, bir adresten yükle ya da gömülü örnek testi dene.
- **Paylaşılabilir link:** `https://<adres>/?quiz=<json-adresi>` testi açılışta otomatik yükler. Raw GitHub veya gist adresleri CORS'a izin verdiği için doğrudan çalışır.
- **İki mod**
  - *Antrenman:* her cevaptan sonra doğru şık ve açıklama gösterilir, süre baskısı yoktur.
  - *Sınav simülasyonu:* geri sayım, cevap değiştirme, "sonra bak" işareti; sonuçlar en sonda. Süre dolunca test kendiliğinden biter.
- **Soru ve şık karıştırma.** Karıştırma, tekrar üretilebilir olsun diye tohumlu (seeded) bir PRNG ile yapılır. `lock_options` işaretli sorularda ("Hepsi", "Yalnızca I ve II" gibi) şıkların sırası korunur.
- **Şık eleme:** sağ tık, uzun basma veya ⊘ düğmesiyle şıkkın üstü çizilir.
- **Olumsuz kök vurgusu:** soru kökündeki "değildir", "yanlıştır", "söylenemez", "NOT" gibi kelimeler otomatik olarak işaretlenir.
- **Sonuç ekranı:** skor, doğru/yanlış/boş sayıları, süre, zayıftan güçlüye konu kırılımı, filtrelenebilir gözden geçirme listesi ve **yanlış ve boşları tekrar çöz** düğmesi.
- **Kaldığı yerden devam:** deneme her değişiklikte `localStorage`'a yazılır, sayfa yenilense de kaybolmaz. Her testin son 10 denemesi saklanır.
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

Metin alanlarında Markdown kullanılabilir: kod blokları, tablolar, listeler, görseller, `$...$` formüller (şimdilik kod olarak gösterilir). İçerikteki ham HTML çalıştırılmaz, metin olarak gösterilir. Bilinmeyen alanlar yok sayılır.

Dosyadaki hatalar soru kimliğiyle birlikte anlaşılır bir mesajla gösterilir (geçersiz JSON'da satır/sütun bilgisiyle).

## Mimari

```
src/
├── domain/       Saf model, tarayıcı API'si yok → cargo test ile test edilir
│   ├── quiz.rs       Quiz, Question, Choice; AnswerKey ayrı tutulur
│   ├── attempt.rs    Deneme durumu: sıra, cevaplar, işaretler, süre, kurallar
│   ├── summary.rs    Skor ve konu kırılımı
│   └── rng.rs        Tohumlu karıştırma
├── dto/          Dosya formatları (v1) → domain dönüşümü ve doğrulama
├── source/       QuizSource trait'i: dosya, URL, gömülü/saklanan metin
├── grading/      Grader trait'i: LocalGrader (cevap anahtarıyla, tarayıcıda)
├── markdown.rs   Güvenli Markdown render, görsel yolları, olumsuz kök vurgusu
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

> **Not:** Faz 1'de cevaplar JSON dosyasında açıkça durur. Uygulama antrenman amaçlı olduğu için bu bilinçli bir karardır; cevapları gizlemeye çalışmak yerine Faz 3'te değerlendirme servise taşınacak.

## Yol haritası

- **Faz 2:** kod renklendirme, KaTeX ile formüller, Exam Markdown → JSON dönüştürücü, konu bazlı ilerleme geçmişi.
- **Faz 3:** `ApiSource` + `RemoteGrader`, servis tarafı, öğrenci kimliği ve sonuçların toplanması.

## Yayınlama

`main` dalına yapılan her push'ta GitHub Actions testleri çalıştırır ve uygulamayı GitHub Pages'e yayınlar ([`.github/workflows/deploy.yml`](.github/workflows/deploy.yml)). İlk seferde repo ayarlarından **Settings → Pages → Source: GitHub Actions** seçilmelidir.
