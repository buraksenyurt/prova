# Tasarım Kalıpları: Senaryolar

<!--
farm-java-dp-scenarios reposundaki senaryolara dayanan, ezbere değil muhakemeye dayalı
10 soru. Kaynak (açıklamasız) sürüm farm-java-dp-scenarios reposundaki java-exams-02.md dosyasıdır.
Açıklamalar şıklara harfle atıf yaptığı için her soruda "lock" yorumu ile şık sırası sabitlenmiştir.
-->

<!-- id: java-02 -->

Tasarım kalıplarını **ne zaman kullanmalı, ne zaman kullanmamalı** sorusu üzerinden ele alan 10 senaryo sorusu *(Strategy, State, Chain of Responsibility, Decorator, Adapter, Facade, Command)*. Kalıp sırasının davranışa etkisi, sorumluluğun nerede durması gerektiği ve benzer görünen kalıpların farkı gibi konular Java 21 kod parçalarıyla değerlendirilir. Antrenman modunda her cevaptan sonra açıklamayı okumayı unutma; açıklamalar diğer şıkların neden yanlış olduğunu da anlatır.

## Soru 1
<!-- topics: Design Patterns, Strategy -->
<!-- lock -->

Bir e-ticaret uygulamasında kargo ücreti iki kurala göre hesaplanıyor: yurt içi gönderiler için sabit **49,90 TL**, yurt dışı gönderiler için sabit **199,90 TL**. İş birimi bu yapının önümüzdeki yıllarda değişmesini beklemediğini açıkça belirtmiş. Mevcut kod şu şekilde:

```java
public double shippingFee(Shipment shipment) {
    return shipment.isDomestic() ? 49.90 : 199.90;
}
```

Ekibe yeni katılan bir geliştirici, kodu **Strategy** kalıbına taşımak için bir `ShippingStrategy` arayüzü, `DomesticShipping` ve `InternationalShipping` sınıfları ile bu stratejileri tutan bir `EnumMap` içeren bir pull request açıyor. Code review sırasında yazılabilecek en isabetli yorum hangisidir?

- A) Değişiklik yerindedir. Kod içinde koşullu ifade *(if / ternary)* kalması her durumda **Open/Closed Principle** ihlalidir ve Strategy kalıbı ile giderilmelidir.
- B) Mevcut kod yeterlidir. Strategy kalıbı aynı iş için birden fazla ve değişmesi beklenen algoritma olduğunda, yeni kuralların sık eklendiği veya kuralların bağımsız test edilmesi gerektiği durumlarda değer üretir. İki dallı ve değişmesi beklenmeyen bir kural için arayüz ve sınıflar eklemek kodu sadeleştirmez, dağıtır.
- C) Strategy yerine **State** kalıbı kullanılmalıdır. Çünkü gönderinin yurt içi ya da yurt dışı olması, nesnenin bir durumudur.
- D) Strategy yerine **Chain of Responsibility** kullanılmalıdır. Böylece yurt içi kuralı uygulanamazsa talep yurt dışı kuralına devredilir.

> Açıklama: Kalıplar bir sorunu çözmek için vardır; sorun yoksa kalıp da maliyettir. Strategy'nin kazancı **değişkenliği** yönetmesidir: Algoritma sayısı artıyorsa, çalışma zamanında seçiliyorsa, kampanya gibi nedenlerle sık değişiyorsa ya da her kuralın ayrı test edilmesi gerekiyorsa arayüz arkasına almak yatırımını geri öder. Burada iki sabit değer ve değişmeyecek bir kural var. Üç yeni tip ve bir harita eklemek okuyucunun takip etmesi gereken yer sayısını artırır *(YAGNI - You Aren't Gonna Need It)*.
>
> - **A** yanlış: OCP, koşullu ifadeleri yasaklamaz. Değişmesi beklenen yerlerin genişletilebilir olmasını ister. Değişmeyecek bir kural için soyutlama eklemek *spekülatif genellik* olarak bilinen bir kod kokusudur.
> - **C** yanlış: State, nesnenin **davranışının zaman içinde değişen durumuna göre** farklılaştığı ve durumlar arasında geçiş olduğu senaryolar içindir. Bir gönderinin yurt içi olması bir geçiş içermez, sabit bir özelliktir.
> - **D** yanlış: Chain of Responsibility sırayla çalışan ve akışı kesebilen işleyiciler içindir. Burada iki alternatiften biri seçilmektedir; zincir kurmak ilişkiyi yanlış modeller.

## Soru 2
<!-- topics: Design Patterns, Strategy, State -->
<!-- lock -->

Bir yazılım şirketi iki farklı modül üzerinde çalışmaktadır.

- **Faturalandırma modülü:** Kullanıcı ödeme sayfasında *aylık*, *yıllık* veya *kullandıkça öde* planlarından birini seçer. Seçilen plana göre tutar farklı bir algoritma ile hesaplanır. Planlar birbirinden habersizdir ve kullanıcı istediği zaman başka bir plana geçebilir.
- **Doküman onay modülü:** Bir doküman **Taslak -> İncelemede -> Onaylandı / Reddedildi** şeklinde ilerler. Taslaktaki doküman düzenlenebilir ama onaylanamaz, incelemedeki doküman düzenlenemez ama onaylanabilir. Bir işlem tamamlandığında dokümanın hangi duruma geçeceğine dokümanın o anki durumu karar verir.

Bu iki modül için hangi tasarım kalıpları en uygundur?

- A) Her iki modül için de **Strategy**
- B) Her iki modül için de **State**
- C) Faturalandırma için **Strategy**, doküman onayı için **State**
- D) Faturalandırma için **State**, doküman onayı için **Strategy**

> Açıklama: Strategy ve State kalıplarının sınıf diyagramları neredeyse aynıdır: Bir arayüz, onu uygulayan sınıflar ve işi bu arayüze devreden bir context. Fark **niyettedir** ve iki soruyla ayırt edilir: *Hangi davranışın çalışacağını kim seçiyor?* ve *Seçenekler birbirini tanıyor mu?*
>
> - Faturalandırmada planı **istemci** *(kullanıcı)* seçer ve planlar birbirinden habersizdir. Bu Strategy'dir.
> - Doküman onayında geçişe **durumun kendisi** karar verir; "İncelemede" durumu onay geldiğinde "Onaylandı" durumuna geçileceğini bilir. Durumlar birbirini tanır. Bu State'tir.
>
> **A** ve **B** iki modülü aynı kalıba zorlar; **D** ise iki kalıbın rollerini ters çevirir. Faturalandırmada plan değişikliği bir "durum geçişi" değil, istemcinin farklı bir algoritma seçmesidir.

## Soru 3
<!-- topics: Design Patterns, State -->
<!-- lock -->

Sipariş yaşam döngüsü **State** kalıbı ile modellenmiştir. Her durum ayrı bir sınıftır ve geçersiz işlemler arayüzdeki varsayılan metotlarla reddedilir.

```java
public interface OrderState {
    String name();
    default void pay(Order order)     { throw invalid("ödenemez"); }
    default void ship(Order order)    { throw invalid("kargoya verilemez"); }
    default void cancel(Order order)  { throw invalid("iptal edilemez"); }

    private IllegalStateException invalid(String action) {
        return new IllegalStateException(name() + " durumundaki sipariş " + action);
    }
}

public class Order {
    private OrderState state = new NewState();

    public void pay()    { state.pay(this); }
    public void ship()   { state.ship(this); }
    public void cancel() { state.cancel(this); }

    void transitionTo(OrderState next) { this.state = next; }
    public String getStatus() { return state.name(); }
}
```

Mevcut durumlar `NewState`, `PaidState`, `ShippedState` ve `CancelledState` sınıflarıdır. *(Bu soru için teslim adımı ve olay geçmişi kod kesitinden çıkarılmıştır.)* Yeni bir iş kuralı gelir: *"Kargoya verilmiş bir sipariş için müşteri iade talebi oluşturabilir. Sipariş bu durumda RETURN_REQUESTED durumuna geçer. Diğer durumlarda iade talebi reddedilir."* Bu değişiklik için aşağıdaki adımlar önerilmiştir.

- **I.** `RETURN_REQUESTED` durumunu temsil eden yeni bir `ReturnRequestedState` sınıfı yazılır.
- **II.** `OrderState` arayüzüne varsayılan olarak exception fırlatan bir `requestReturn(Order order)` metodu, `Order` sınıfına da işi duruma devreden bir `requestReturn()` metodu eklenir.
- **III.** `ShippedState` sınıfı `requestReturn` metodunu ezerek *(override)* siparişi `ReturnRequestedState` durumuna geçirir.
- **IV.** `NewState`, `PaidState` ve `CancelledState` sınıflarının her birine, iade talebini reddetmek için exception fırlatan bir `requestReturn` metodu eklenir.

Bu iş kuralını uygulamak için hangi adımlar gerekli ve yeterlidir?

- A) I ve III
- B) Hepsi
- C) II ve IV
- D) I, II ve III

> Açıklama: Varsayılan *(default)* metotlar bu tasarımın en önemli kazancıdır: "Bu durumda bu işlem yapılamaz" davranışı tek bir yerde tanımlıdır. Her durum sınıfı yalnızca **kendisi için geçerli** olan işlemleri ezer.
>
> - **I** gerekli: Yeni bir durum, yeni bir sınıf demektir.
> - **II** gerekli: Bu bir yeni **durum** olduğu kadar yeni bir **işlemdir** de. İşlem arayüze eklenmezse durumlar onu ezemez; `Order`'a eklenmezse istemci onu çağıramaz. Sadece yeni bir durum eklenip yeni işlem eklenmeseydi `Order` sınıfı hiç değişmezdi.
> - **III** gerekli: Geçişe karar veren, işlemin geçerli olduğu tek durumdur.
> - **IV** gereksiz: Arayüzdeki varsayılan metot zaten exception fırlatır. Diğer sınıflara aynı kodu eklemek, kalıbın ortadan kaldırdığı tekrarı geri getirir.
>
> Bu yüzden **A** eksik *(II olmadan işlem çağrılamaz)*, **B** ve **C** ise gereksiz tekrar içerir *(IV)*.

## Soru 4
<!-- topics: Design Patterns, Chain of Responsibility -->
<!-- lock -->

Bir bankanın havale/EFT modülünde her transfer talebi **Chain of Responsibility** kalıbıyla kurulmuş bir zincirden geçmektedir. Herhangi bir halka talebi reddederse akış durur. Halkalar ve maliyetleri şöyledir:

| Halka | Ne kontrol eder? | Maliyet |
| --- | --- | --- |
| `RequestFormatCheck` | IBAN biçimi, tutarın pozitif olması | Bellekte, mikro saniyeler |
| `DescriptionRequiredCheck` | 50.000 TL ve üzerindeki transferlerde açıklamanın boş olup olmadığı | Bellekte, mikro saniyeler |
| `BalanceCheck` | Gönderen hesabın bakiyesi | Veritabanı sorgusu |
| `BlacklistCheck` | Alıcının kara listede olup olmadığı | Uzak bir dolandırıcılık servisi, ~300 ms, çağrı başına ücretli |

Mevcut zincir `BlacklistCheck -> BalanceCheck -> RequestFormatCheck -> DescriptionRequiredCheck` sırasıyla kurulmuştur. Loglar, reddedilen taleplerin büyük çoğunluğunun hatalı IBAN veya eksik açıklama nedeniyle reddedildiğini gösteriyor. Halkalar birbirinden bağımsızdır ve iş birimi, bir talep birden fazla kuralı ihlal ettiğinde hangi ret gerekçesinin döndüğünü önemsemediğini belirtmiştir.

Ölçülen ret nedenleri ve belirtilen çağrı maliyetleri dikkate alındığında, aşağıdaki sıralamalardan hangisi gereksiz veritabanı ve uzak servis çağrılarını azaltmak için en uygundur?

- A) `RequestFormatCheck -> DescriptionRequiredCheck -> BalanceCheck -> BlacklistCheck`
- B) `BlacklistCheck -> RequestFormatCheck -> DescriptionRequiredCheck -> BalanceCheck`, çünkü güvenlik kontrolleri her zaman ilk sırada olmalıdır.
- C) Sıra önemli değildir. Zincirdeki tüm halkalar her talep için mutlaka çalışır.
- D) `BalanceCheck -> BlacklistCheck -> RequestFormatCheck -> DescriptionRequiredCheck`, çünkü bakiyesi yetersiz talepler en sık karşılaşılan durumdur.

> Açıklama: Zincirde bir halka reddettiğinde sonraki halkalar **hiç çalışmaz**. Bu yüzden sıra bir iş kararı olduğu kadar bir **maliyet** kararıdır da: Ucuz ve dış kaynağa gitmeyen kontroller öne, pahalı olanlar *(veritabanı, uzak servis)* sona konur. Böylece hatalı talepler pahalı halkalara hiç ulaşmaz. **A**, sık ret üreten iki bellek içi kontrolü de dış çağrıların önüne alan tek seçenektir. *(Verilen bilgiler, kalan taleplerde bakiye ile kara liste kontrolünden hangisinin önce çalışmasının mutlak en düşük maliyeti vereceğini belirlemeye yetmez; soru da bunu sormaz.)*
>
> - **B** yanlış: Senaryoda halkalar bağımsız ve ret gerekçesinin önceliği önemsenmiyor. Her talep için ücretli bir uzak çağrıyı en başta yapmak, zaten biçim hatasıyla reddedilecek talepler için de ödeme yapmak demektir. Güvenlik kontrolünün sonda olması onu atlamaz; onaylanan her talep yine kara liste kontrolünden geçer.
> - **C** yanlış: Bu kalıbın tanımı gereği bir halka akışı kesebilir. Tüm kontrollerin her zaman çalıştığı bir yapı zincir değil, bir doğrulama listesi olurdu.
> - **D** yanlış: Senaryodaki veri, retlerin çoğunun biçim ve açıklama hatalarından geldiğini söylüyor. Üstelik veritabanı ve uzak servis çağrılarını öne almak maliyeti artırır.

## Soru 5
<!-- topics: Design Patterns, Chain of Responsibility -->
<!-- lock -->

Bir bankada transfer kontrolleri **Chain of Responsibility** kalıbıyla kurulmuştur. `TransferService`, yapıcı metot üzerinden zincirin ilk halkasını alır ve hangi kontrollerin çalıştığını bilmez.

```java
public class TransferService {
    private final TransferCheck firstCheck;

    public TransferService(TransferCheck firstCheck) {
        this.firstCheck = firstCheck;
    }

    public TransferResult validate(TransferRequest request) {
        return firstCheck.check(request);
    }
}
```

Yeni gereksinimler şöyledir: **Mobil kanalın** zinciri beş kontrolün tamamını *(biçim, kara liste, bakiye, günlük limit, açıklama)* içermeli ve doğrulama ilk rette sonlanmalıdır. **Şube kanalında** günlük limit uygulanmamalı, diğer kontroller aynen kalmalıdır. **Kurumsal kanalda** ise mobil kanaldaki kontrollere ek olarak ikinci imza kontrolü yapılmalıdır.

Bu gereksinimleri karşılamak için en uygun yaklaşım hangisidir?

- A) `DailyLimitCheck` sınıfının içine `if (request.channel() == Channel.BRANCH) return Optional.empty();` gibi bir kanal kontrolü eklemek.
- B) `TransferService` sınıfına `skipDailyLimit` ve `requireSecondSignature` gibi **boolean** parametreler eklemek ve zinciri bu parametrelere göre servis içinde kurmak.
- C) Aynı halka sınıflarını farklı sıralar ve kombinasyonlarla bağlayan üç ayrı zincir tanımlamak *(örneğin `TransferChecks.mobile()`, `branch()`, `corporate()`)*, kurumsal kanal için yeni bir `SecondSignatureCheck` halkası yazmak ve her kanala ait `TransferService` örneğine ilgili zincirin ilk halkasını vermek. Her zincir kendi halka örneklerini oluşturur.
- D) Her kanal için `TransferService` sınıfından türeyen bir alt sınıf yazıp tüm kontrolleri bu alt sınıfların içinde iç içe **if** blokları ile yeniden kodlamak.

> Açıklama: Chain of Responsibility'nin kazançlarından biri, halkaların zincirden **bağımsız** olmasıdır: Bir halka yalnızca kendi kuralını bilir, kimin önünde ya da arkasında durduğunu bilmez. Bu sayede aynı halkalar farklı kombinasyonlarla birleştirilerek kanala özgü zincirler kurulabilir. Yeni bir kural *(ikinci imza)* yeni bir halka sınıfı demektir; mevcut halkalara ve servise dokunulmaz. Bir ayrıntı: Halkalar sonraki halkanın referansını kendi içinde tuttuğu için *(`linkWith`)* aynı halka **nesnesi** iki zincirde paylaşılamaz; bu yüzden her zincir kendi örneklerini oluşturur.
>
> - **A** yanlış: Günlük limit kuralının kendisi kanal bilgisine bağımlı hale gelir. Her yeni kanal bu sınıfın değişmesini gerektirir ve zincirin kurulumunda görünmesi gereken bir karar halkanın içine gizlenir.
> - **B** yanlış: Boolean bayraklar zincirin kurulum sorumluluğunu servise taşır. Kanal ve kural sayısı arttıkça bayrak kombinasyonları çoğalır ve servis her yeni kuralda değişir.
> - **D** yanlış: Kalıbın ortadan kaldırdığı iç içe if bloklarını ve kod tekrarını geri getirir.

## Soru 6
<!-- topics: Design Patterns, Decorator -->
<!-- lock -->

Döviz kuru getiren bir servise önbellekleme, loglama ve yeniden deneme yetenekleri **Decorator** kalıbı ile eklenmiştir. Üç dekoratör de `ExchangeRateProvider` arayüzünü uygular ve kurucu metotlarında sarmaladıkları nesneyi *(delegate)* alır.

```java
public interface ExchangeRateProvider {
    double getRate(String currencyPair);
}

public class CachingRateProvider implements ExchangeRateProvider {
    private final ExchangeRateProvider delegate;
    private final Map<String, Double> cache = new ConcurrentHashMap<>();

    public CachingRateProvider(ExchangeRateProvider delegate) { this.delegate = delegate; }

    @Override
    public double getRate(String currencyPair) {
        return cache.computeIfAbsent(currencyPair, delegate::getRate);
    }
}

public class LoggingRateProvider implements ExchangeRateProvider {
    private final ExchangeRateProvider delegate;
    private final Consumer<String> log;

    public LoggingRateProvider(ExchangeRateProvider delegate, Consumer<String> log) {
        this.delegate = delegate;
        this.log = log;
    }

    @Override
    public double getRate(String currencyPair) {
        log.accept(currencyPair + " isteniyor");
        try {
            double rate = delegate.getRate(currencyPair);
            log.accept(currencyPair + " = " + rate);
            return rate;
        } catch (RuntimeException e) {
            log.accept(currencyPair + " alınamadı: " + e.getMessage());
            throw e;
        }
    }
}

public class RetryingRateProvider implements ExchangeRateProvider {
    private final ExchangeRateProvider delegate;
    private final int maxAttempts;

    public RetryingRateProvider(ExchangeRateProvider delegate, int maxAttempts) {
        if (maxAttempts < 1) {
            throw new IllegalArgumentException("maxAttempts en az 1 olmalı");
        }
        this.delegate = delegate;
        this.maxAttempts = maxAttempts;
    }

    @Override
    public double getRate(String currencyPair) {
        IllegalStateException lastError = null;
        for (int attempt = 1; attempt <= maxAttempts; attempt++) {
            try {
                return delegate.getRate(currencyPair);
            } catch (IllegalStateException e) {
                lastError = e;
            }
        }
        throw lastError;
    }
}
```

`RemoteRateClient(2)` nesnesi uzak servisi temsil eder. İlk **2** çağrıda `IllegalStateException` *(zaman aşımı)* fırlatır, sonraki çağrılarda kuru döner. `getCallCount()` uzak servise kaç kez gidildiğini verir. `computeIfAbsent` çağrısında hesaplama exception ile biterse önbelleğe kayıt eklenmez. Bir geliştirici dekoratörleri aşağıdaki sırayla birleştiriyor.

```java
RemoteRateClient client = new RemoteRateClient(2);
List<String> log = new ArrayList<>();

ExchangeRateProvider provider =
        new RetryingRateProvider(
                new LoggingRateProvider(
                        new CachingRateProvider(
                                new RemoteExchangeRateProvider(client)),
                        log::add),
                3);

provider.getRate("USD/TRY");
provider.getRate("USD/TRY");

System.out.println(client.getCallCount() + " / " + log.size());
```

Sizce bu kodun ekran çıktısı ne olur?

- A) `3 / 8`
- B) `3 / 4`
- C) `1 / 2`
- D) `5 / 8`

> Açıklama: Dekoratörlerin sırası davranışı doğrudan belirler. Burada en dışta **Retrying**, onun içinde **Logging**, en içte **Caching** var. Yani her yeniden deneme Logging katmanından geçer.
>
> **Birinci çağrı:**
> - 1. deneme: Logging `isteniyor` yazar. Cache boş, uzak servis hata verir *(1. uzak çağrı)*. Hesaplama exception ile bittiği için cache'e kayıt eklenmez. Logging `alınamadı` yazar.
> - 2. deneme: Aynı akış *(2. uzak çağrı)*, iki log satırı daha.
> - 3. deneme: `isteniyor`, uzak servis başarılı olur *(3. uzak çağrı)*, değer cache'e girer, Logging sonucu yazar.
> - Toplam: 3 uzak çağrı, 6 log satırı.
>
> **İkinci çağrı:** Logging `isteniyor` yazar, değer cache'ten gelir *(uzak çağrı yok)*, Logging sonucu yazar. 2 log satırı.
>
> Sonuç **3 uzak çağrı, 8 log satırı**. **B** *(`3 / 4`)*, Logging en dıştayken alınacak sonuçtur; o dizilimde başarısız denemeler Retrying'in içinde kaldığı için loglanmaz. **C** cache'in hata almış bir hesaplamayı saklayacağını varsayar. **D** ise ikinci çağrının da uzak servise gittiğini varsayar.

## Soru 7
<!-- topics: Design Patterns, Decorator -->
<!-- lock -->

Bir döviz kuru servisine önbellek yeteneği **Decorator** kalıbıyla eklenmiştir. `CachingRateProvider`, bir kur çiftini ilk kez aldığında değeri uygulama çalıştığı sürece saklamaktadır.

```java
ExchangeRateProvider provider =
        new CachingRateProvider(
                new RetryingRateProvider(
                        new RemoteExchangeRateProvider(client), 3));
```

Uygulama günlerce yeniden başlatılmadan çalıştığı için kullanıcılar sabahki kuru akşam da görmeye başlamıştır. İş birimi, önbellekte **5 dakikasını dolduran** bir kurun bir sonraki sorguda uzak servisten yenilenmesini istiyor. Aynı modülü kullanan **raporlama** ekibi ise her çalıştırmada tek bir kapanış gününe ait sabit kurlarla çalıştığı için mevcut, süresiz önbellekten memnundur. İstemci kodlarının `ExchangeRateProvider` arayüzü dışında hiçbir şey bilmemesi isteniyor.

Bu ihtiyacı karşılamak için en uygun yaklaşım hangisidir?

- A) `CachingRateProvider` sınıfına `boolean expiring` ve `Duration ttl` parametreleri eklemek ve metodun içinde bu bayrağa göre iki farklı önbellek davranışı uygulamak.
- B) Önbelleği tamamen kaldırıp her çağrıda uzak servise gitmek. Böylece kur her zaman güncel olur.
- C) `RemoteExchangeRateProvider` sınıfından türeyen `ExpiringRemoteExchangeRateProvider` ve `CachingRemoteExchangeRateProvider` gibi alt sınıflar yazmak ve her yetenek kombinasyonu için yeni bir alt sınıf oluşturmak.
- D) Aynı arayüzü uygulayan ve süreyi dışarıdan alan *(saat bilgisini de test edilebilirlik için `java.time.Clock` olarak enjekte eden)* yeni bir `ExpiringCachingRateProvider` dekoratörü yazmak. Kur ekranında bu dekoratörü, raporlamada mevcut `CachingRateProvider`'ı kullanmak. Hangisinin kullanılacağına nesneler oluşturulurken karar vermek.

> Açıklama: Decorator kalıbında her yetenek, aynı arayüzü uygulayan ayrı bir sınıftır. Yeni bir davranış *(süreli önbellek)* yeni bir dekoratör demektir. Hangi modülün hangi dekoratörü kullanacağına **nesne grafiği kurulurken** *(composition root)* karar verilir; istemci kodu ve mevcut dekoratörler değişmez. Saatin `Clock` olarak dışarıdan verilmesi, testte 5 dakika beklemeden zamanı ileri sarmayı mümkün kılar.
>
> - **A** yanlış: Boolean bayraklar, Decorator kalıbının ortadan kaldırdığı sorunun ta kendisidir. İki farklı sorumluluk tek bir sınıfa dolanır ve her yeni önbellek politikası bu sınıfı değiştirir.
> - **B** yanlış: Önbelleğin var olma nedenini *(uzak servisin yavaş ve ücretli olması)* görmezden gelir. Ayrıca raporlama ekibinin ihtiyacını da bozar.
> - **C** yanlış: Kalıtım ile yetenek eklemek, her kombinasyon için yeni bir alt sınıf gerektirir *(sınıf patlaması)*. Önbellek, log, retry gibi yetenekler arttıkça kombinasyon sayısı katlanarak büyür. Ayrıca yeteneklerin sırası çalışma zamanında değiştirilemez.

## Soru 8
<!-- topics: Design Patterns, Adapter -->
<!-- lock -->

Bir e-ticaret uygulaması iki sanal POS sağlayıcısı ile **Adapter** kalıbı üzerinden çalışmaktadır. İş kodu yalnızca `PaymentGateway` arayüzünü tanır. Aşağıdaki varsayımsal ilk sürümde tutar `double` olarak alınmakta ve her adapter onu kendi formatına kendisi çevirmektedir.

```java
public interface PaymentGateway {
    PaymentResult charge(String orderNo, double amount);
}

// PayFast SDK'sı tutarı kuruş cinsinden long bekler
public class PayFastAdapter implements PaymentGateway {
    public PaymentResult charge(String orderNo, double amount) {
        long kurus = Math.round(amount * 100);
        // client.makePayment(orderNo, kurus, "TRY") ...
    }
}

// Banka kütüphanesi tutarı virgüllü metin olarak bekler
public class LegacyBankPosAdapter implements PaymentGateway {
    public PaymentResult charge(String orderNo, double amount) {
        String text = String.format(Locale.ROOT, "%.2f", amount).replace('.', ',');
        // pos.process("<amount>" + text + "</amount> ...") ...
    }
}
```

Bir gün **1,005 TL** tutarlı bir test siparişinde PayFast'in **100 kuruş** *(1,00 TL)*, bankanın ise **1,01 TL** çektiği fark ediliyor. *(`1.005` değeri double olarak `1.00499...` şeklinde tutulduğu için `Math.round(1.005 * 100)` ifadesi `100`, `String.format(Locale.ROOT, "%.2f", 1.005)` ifadesi ise `"1.01"` sonucunu üretir.)*

Ödeme sözleşmesine göre tutar pozitif olmalı ve tam kuruş olarak ifade edilebilmelidir. Kuruşun kesrini içeren talepler herhangi bir sağlayıcı çağrılmadan reddedilmelidir. Bu sözleşmeyi tüm sağlayıcılar için tek yerde hangi tasarım uygular?

- A) Her iki adapter'da da aynı yuvarlama yöntemini kullanmak, örneğin bankada da `Math.round(amount * 100)` ile kuruşa çevirip metne dönüştürmek.
- B) Tutarı sistem sınırında *(örneğin `CheckoutService` içinde)* bir kez doğrulayıp kuruş cinsinden tutan bir değer nesnesine *(`Amount` gibi bir value object)* çevirmek, kuruştan küçük hassasiyetli tutarları yuvarlamak yerine reddetmek ve `PaymentGateway` arayüzünün bu değer nesnesini almasını sağlamak. Adapter'lar artık yalnızca format dönüşümü yapar.
- C) `double` yerine `float` kullanmak. Daha az basamak tutulduğu için yuvarlama farkları ortadan kalkar.
- D) Adapter kalıbını kaldırıp iki sağlayıcının çağrılarını `CheckoutService` içinde `if-else` ile yapmak. Böylece tüm dönüşümler tek bir sınıfta toplanır.

> Açıklama: Sorunun kökü, **parasal bir karar** *(yuvarlama, geçerlilik)* her adapter'ın içinde ayrı ayrı veriliyor olmasıdır. Adapter'ın görevi bir arayüzü başka bir arayüze **dönüştürmektir**; iş kuralı koymak değil. Tutar sistem sınırında bir kez doğrulanıp bir değer nesnesine çevrildiğinde iki sağlayıcı da **aynı** parasal değeri alır ve hatalı hassasiyet hiçbir sağlayıcıya ulaşmaz.
>
> - **A** iki sağlayıcıyı tutarlı hâle getirir *(ikisi de 100 kuruş çeker)* ama sözleşmeye göre reddedilmesi gereken geçersiz bir ödemeyi **kabul eder**; kararı da yine adapter'larda bırakır. 1,005 TL'lik bir talep hata vermeden 1,00 TL olarak çekilir. Üçüncü bir adapter yazan geliştiricinin aynı yöntemi kullanacağının da garantisi yoktur.
> - **C** yanlış: `float` daha **az** hassastır; ikilik tabanda tam ifade edilemeyen ondalık değerler sorunu büyür.
> - **D** yanlış: Adapter kalıbını kaldırmak, iş kodunun üçüncü parti API detaylarını yeniden bilmesine yol açar. Sorun kalıpta değil, parasal kararın yanlış yerde verilmesindedir.

## Soru 9
<!-- topics: Design Patterns, Adapter -->
<!-- lock -->

Bir şirketin iki farklı ekibi iki farklı entegrasyon problemiyle uğraşmaktadır.

- **Fatura ekibi:** Fatura PDF'i üretmek için üçüncü parti bir PDF kütüphanesi kullanıyor. Basit bir faturayı oluşturmak için bile kütüphanenin `Document`, `PageLayout`, `FontRegistry`, `TableBuilder`, `Renderer` gibi on iki farklı sınıfının doğru sırayla kullanılması gerekiyor. Ekip, uygulamanın geri kalanının sadece `invoicePdf.render(invoice)` gibi tek bir metot görmesini istiyor.
- **Ödeme ekibi:** Uygulama zaten `PaymentGateway` adında kendi arayüzünü tanımlamış ve tüm iş kodu bu arayüzle çalışıyor. Yeni anlaşılan bir sağlayıcının SDK'sı tutarı farklı birimde, sonucu farklı hata kodlarıyla döndürüyor. Ekip, bu SDK'nın mevcut `PaymentGateway` arayüzüne uymasını istiyor.

Bu iki ihtiyaç için hangi kalıplar en uygundur?

- A) Fatura ekibi için **Facade**, ödeme ekibi için **Adapter**
- B) Fatura ekibi için **Adapter**, ödeme ekibi için **Facade**
- C) Her iki ekip için de **Adapter**
- D) Fatura ekibi için **Decorator**, ödeme ekibi için **Adapter**

> Açıklama: Facade ile Adapter sıklıkla karıştırılır çünkü ikisi de bir alt sistemin önüne konan bir sarmalayıcıdır. Fark, **hedef arayüzün nereden geldiğidir**:
>
> - **Facade**, karmaşık bir alt sistemi **sadeleştirmek** için **yeni** ve daha basit bir arayüz tanımlar. Fatura ekibinin istediği, on iki sınıfı tek bir metodun arkasına saklamaktır; ortada uyulması gereken önceden tanımlı bir arayüz yoktur.
> - **Adapter**, **var olan** bir arayüzü istemcinin **beklediği başka bir arayüze** dönüştürür. Ödeme ekibinde hedef arayüz *(`PaymentGateway`)* zaten bellidir; yeni SDK ona uydurulur.
>
> **B** iki kalıbın rollerini ters çevirir. **C**, fatura tarafında dönüştürülecek bir hedef arayüz olmadığı için yanlıştır. **D** yanlıştır, çünkü Decorator aynı arayüzü koruyarak bir nesneye yeni yetenek ekler; karmaşıklığı gizlemek için kullanılmaz.

## Soru 10
<!-- topics: Design Patterns, Command -->
<!-- lock -->

Sıra tabanlı bir strateji oyununda hamlelerin geri alınabilmesi için **Command** kalıbı kullanılmıştır. Oyun tahtası bir birimin can puanını asla sıfırın altına düşürmez. *(`Command` arayüzünün `execute` ve `undo` dışındaki üyeleri kesitte gösterilmemiştir.)*

```java
public class GameBoard {
    private final Map<String, Integer> healthPoints = new HashMap<>();

    public int hpOf(String unit) { return healthPoints.get(unit); }

    public void setHp(String unit, int hp) {
        healthPoints.put(unit, Math.max(0, hp));
    }
    // diğer metotlar...
}

public class AttackCommand implements Command {
    private final GameBoard board;
    private final String target;
    private final int damage;

    public AttackCommand(GameBoard board, String target, int damage) {
        this.board = board;
        this.target = target;
        this.damage = damage;
    }

    @Override
    public void execute() {
        board.setHp(target, board.hpOf(target) - damage);
    }

    @Override
    public void undo() {
        board.setHp(target, board.hpOf(target) + damage);
    }
}
```

Canı **20** olan bir birime **35** hasarlık bir saldırı yapılıyor *(`execute`)*, ardından oyuncu bu hamleyi geri alıyor *(`undo`)*. Geri alma işleminden sonra birimin canı kaç olur ve doğru tasarım ne olmalıdır?

- A) Can **20** olur. Mevcut tasarım doğrudur, değişiklik gerekmez.
- B) Can **35** olur. Hedefin kalan canı sıfırın altına düşemediği için "hasar kadar geri ekle" yaklaşımı yanlış sonuç verir. Komut, `execute` sırasında hedefin önceki can değerini saklamalı ve `undo` bu değeri geri yüklemelidir.
- C) Can **0** olur. Çünkü `undo` metodu ölü bir birim üzerinde çalışmaz.
- D) Can **-15** olur. Bu durumu düzeltmek için `GameBoard` sınıfındaki `Math.max(0, hp)` kontrolü kaldırılmalıdır.

> Açıklama: `execute` sonrasında can `Math.max(0, 20 - 35) = 0` olur. `undo` ise `0 + 35 = 35` hesaplar; birim saldırıdan önceki halinden **daha güçlü** bir şekilde geri gelir. Sorun, geri almanın işlemin **tersini hesaplamaya** dayanmasıdır. Kırpma *(clamping)*, yuvarlama ya da yan etkiler varsa ters işlem orijinal durumu vermez.
>
> Güvenilir yöntem, komutun geri alma için gereken bilgiyi **kendi içinde** saklamasıdır: `execute` başında `targetHpBefore = board.hpOf(target)` alınır, `undo` ise `board.setHp(target, targetHpBefore)` ile tam olarak o değeri geri yükler. Command kalıbının gücü de buradadır: İşlem ve onu geri almak için gereken bilgi aynı nesnede birlikte durur.
>
> - **A** yanlış: Kırpma nedeniyle sonuç 20 değil 35'tir.
> - **C** yanlış: Kodda ölü birimi kontrol eden bir koşul yoktur; `undo` çalışır ve canı 35 yapar.
> - **D** yanlış: Can hiçbir zaman negatif olmaz. Tahtadaki kuralı kaldırmak oyunun kendi kuralını bozar; sorun tahtada değil, komutun geri alma yöntemindedir.

## Cevap Anahtarı

| Soru No | Doğru Cevap |
| ------- | ----------- |
| 1 | B |
| 2 | C |
| 3 | D |
| 4 | A |
| 5 | C |
| 6 | A |
| 7 | D |
| 8 | B |
| 9 | A |
| 10 | B |
