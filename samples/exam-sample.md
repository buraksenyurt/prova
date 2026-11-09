# Prova Markdown Örneği

Bu dosya, sınavları Markdown ile yazıp Prova'ya doğrudan yükleyebileceğini gösterir. Uygulamaya sürükle-bırak yapabilir ya da `md2quiz` ile JSON'a çevirebilirsin.

<!-- id: prova-markdown-ornegi -->
<!-- time: 10 -->

## Soru 1

Aşağıdaki Rust kodunda `total` değişkeni için derleyici ne söyler?

```rust
fn main() {
    let total = 0;
    total += 1;
}
```

- A) Kod derlenir ve `total` 1 olur
- B) `total` değiştirilemez (immutable) olduğu için derleme hatası verir
- C) Çalışma zamanında panic oluşur
- D) Uyarı verir ama derlenir

> Açıklama: Rust'ta değişkenler varsayılan olarak değiştirilemez. `let mut total = 0;` yazılmalıdır.

<!-- topics: Rust -->

## Soru 2

Bir REST API için aşağıdaki ifadeler veriliyor:

- I. `GET` istekleri sunucudaki durumu değiştirmemelidir.
- II. `PUT` idempotent bir işlemdir.
- III. `POST` her zaman `200 OK` döndürmelidir.

Hangileri doğrudur?

- A) Yalnızca I
- B) Yalnızca I ve II
- C) Yalnızca II ve III
- D) Hepsi

> Açıklama: Yeni kaynak oluşturan bir `POST` genellikle `201 Created` döner; III yanlıştır.

<!-- topics: Web API -->

## Soru 3

İki vektörün aynı yönü gösterdiği durumda $\cos(\theta)$ değeri kaçtır?

- A) -1
- B) 0
- C) 1
- D) Tanımsız

<!-- topics: RAG, Matematik -->

## Soru 4

Aşağıdakilerden hangisi bir **sürüm kontrol sistemi değildir**?

- A) Git
- B) Mercurial
- C) Subversion
- D) Docker

<!-- topics: Araçlar -->

## Cevap Anahtarı

| Soru No | Doğru Cevap |
|---------|-------------|
| 1       | B           |
| 2       | B           |
| 3       | C           |
| 4       | D           |
