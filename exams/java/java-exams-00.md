# Alıştırma 00 *(SOLID İlkeleri ve Tasarım Kalıpları)*

Bu testin amacı **SOLID *(Single Responsibility, Open/Closed, Liskov Substitution, Interface Segregation, Dependency Inversion)*** ilkeleri ile kurumsal projelerde sık kullanılan bazı tasarım kalıplarını *(Template Method, Strategy, State, Chain of Responsibility, Decorator, Adapter, Command)* **Java** dili üzerinden, kod parçaları ve senaryolar yardımıyla değerlendirmektir. Testin sonunda cevap anahtarı yer almaktadır.

*Not: Sorular Java 21 *(JDK 21)* sürümü üzerinden hazırlanmıştır.*

---

## Soru 1

Geliştirilmekte olan bir abonelik platformunda yazılım geliştirici olarak görev alıyorsunuz. Müşteri modülünde yeni abonelerin JSON *(JavaScript Object Notation)* formatında gelen istekler üzerinden sisteme kaydedildiği bir senaryo söz konusu. Bu işi üstlenen **SubscriberController** sınıfının ilk sürümü aşağıdaki kod parçasında olduğu gibi yazılıyor.

```java
public class SubscriberController {
    private Database db = new Database();

    public String createSubscriber(String jsonValue) throws IOException {
        ObjectMapper mapper = new ObjectMapper();
        Subscriber subscriber = mapper.readValue(jsonValue, Subscriber.class);
        if (!isValidSubscriber(subscriber)) {
            return "ERROR";
        }
        db.save(subscriber);
        return "SUCCESS";
    }

    private boolean isValidSubscriber(Subscriber subscriber) { /* trim + regex kontrolleri */ }
    private boolean isValidAlphaNumeric(String value) { /* ... */ }
    private boolean isValidEmail(String value) { /* ... */ }
}
```

Bu sınıf JSON dönüşümü, verinin temizlenmesi *(trim)*, doğrulama *(validation)* ve depolama *(persistence)* işlerini tek başına üstlenmektedir. Bu nedenle e-posta kuralında, JSON kütüphanesinde veya veritabanında yapılacak herhangi bir değişiklik bu sınıfın değiştirilmesini de gerektirir. Buna göre **Single Responsibility Principle *(SRP)*** prensibi açısından aşağıdaki düzenlemelerden hangisinin doğru olduğu söylenebilir?

- A) Doğrulama metotlarını **static** hale getirip bir **Utility** sınıfına taşımak, JSON dönüşümü ve veritabanı kaydını controller içinde bırakmak.
- B) **SubscriberController** sınıfını **abstract** yapıp JSON, doğrulama ve kayıt işlemlerini farklı alt sınıflarda **override** etmek.
- C) **SubscriberMapper**, **SubscriberNormalizer**, **SubscriberValidator** ve **SubscriberRepositoryService** gibi her biri tek bir işten sorumlu sınıflar oluşturmak ve controller'ın sadece akışı yönetmesini *(orchestration)* sağlamak.
- D) Tüm **private** metotları **public** yaparak sınıfın birim testlerle daha kolay test edilmesini sağlamak.

## Soru 2

Bir e-ticaret sisteminin müşteri kayıt modülünde **CustomerRegistrationService** sınıfı, kayıt öncesinde müşteri bilgilerini aşağıdaki metot ile kontrol etmektedir.

```java
private boolean isValidCustomer(Customer customer) {
    if (customer.getName() == null || customer.getName().isBlank()) {
        return false;
    }
    customer.setName(customer.getName().trim());
    if (!isValidAlphaNumeric(customer.getName())) {
        return false;
    }
    if (customer.getEmail() == null || customer.getEmail().trim().length() == 0) {
        return false;
    }
    customer.setEmail(customer.getEmail().trim());
    return isValidEmail(customer.getEmail());
}

private boolean isValidEmail(String value) {
    return Pattern.compile("^[\\w.+-]+@[\\w-]+\\.[\\w.]+$").matcher(value).matches();
}
```

Bu metodu tasarım açısından değerlendirdiğinizde dikkat çeken en kritik sorun aşağıdakilerden hangisidir?

- A) Metot adı doğrulama yaptığını söylese de **setName** ve **setEmail** çağrıları ile nesne verisini değiştirmektedir *(side effect)*. Verinin temizlenmesi *(normalization)* ve doğrulanması *(validation)* iki ayrı sorumluluktur ve doğrulama yapan kodun yan etkisinin olmaMAsı beklenir.
- B) **Pattern.compile** çağrısı metot her çalıştığında tekrarlandığı için kod derlenmez.
- C) Metot **private** tanımlandığı için **Customer** nesnesinin setter metotlarına erişemez.
- D) Java'da **boolean** dönen metotlar içinde birden fazla **return** ifadesi kullanılamaz.

## Soru 3

Farklı LLM *(Large Language Model)* sağlayıcılarına abone olan kullanıcıların harcadığı AI kredi harcamalarını hesaplayan bir faturalandırma servisi geliştirdiğimizi düşünelim. Bu servisin içeriği aşağıdaki gibidir. **ClaudeSubscriber**, **GrokSubscriber** ve **LlamaSubscriber** sınıfları arasında hesaplama davranışını tanımlayan ortak bir üst tip *(abstraction)* bulunmaMAktadır.

```java
public class BillingService {
    public double calculateTotalCredit(List<Object> subscribers) {
        double total = 0;
        for (Object subscriber : subscribers) {
            total += calculateCredit(subscriber);
        }
        return total;
    }

    public double calculateCredit(Object subscriber) {
        if (subscriber instanceof ClaudeSubscriber claude) {
            return claude.calculateAiCredit();
        } else if (subscriber instanceof GrokSubscriber grok) {
            return grok.calculateAiCredit();
        } else if (subscriber instanceof LlamaSubscriber llama) {
            return llama.calculateAiCredit();
        }
        throw new IllegalArgumentException("Bilinmeyen abone türü: " + subscriber.getClass().getSimpleName());
    }
}
```

Bu kod parçası ile ilgili aşağıdaki görüşler ortaya atılmıştır;

- **I.** Sisteme **MistralSubscriber** gibi yeni bir abone türü eklendiğinde **BillingService** sınıfına yeni bir **else if** dalı eklemek gerekir. Bu durum **Open/Closed Principle *(OCP)*** ihlalidir.
- **II.** Yeni dalı eklemeyi unutmak derleme zamanında fark edilmez. Hata ancak çalışma zamanında **IllegalArgumentException** olarak ortaya çıkar.
- **III.** Abone sınıfları `calculateAiCredit()` metodunu bildiren ortak bir soyut sınıftan *(abstract class)* türeseydi, servis `List<Subscriber>` üzerinde dolaşarak tip kontrolü yapmadan polimorfik çağrı yapabilirdi.
- **IV.** `List<Object>` kullanımı derleyiciye daha fazla bilgi verdiğinden tip güvenliğini *(type safety)* artırır.

Sizce hangi ifade/ifadeler doğrudur?

- A) Yalnız I
- B) I, II ve III
- C) II ve IV
- D) Hepsi

## Soru 4

Bir raporlama modülünde farklı formatlarda *(PDF, Excel, CSV)* rapor üretilmektedir. Tüm formatlarda raporun verisi aynı şekilde yüklenip filtrelenmekte, yalnızca çıktının oluşturulma biçimi değişmektedir. Bu amaçla aşağıdaki soyut sınıf tasarlanmıştır.

```java
public abstract class ReportExporter {
    protected final ReportRepository repository;

    protected ReportExporter(ReportRepository repository) {
        this.repository = repository;
    }

    public final byte[] export(ReportQuery query) {
        List<ReportRow> rows = repository.load(query).stream()
                .filter(ReportRow::isVisible)
                .toList();
        return render(rows);
    }

    protected abstract byte[] render(List<ReportRow> rows);
}

public class CsvReportExporter extends ReportExporter {
    public CsvReportExporter(ReportRepository repository) { super(repository); }

    @Override
    protected byte[] render(List<ReportRow> rows) {
        return rows.stream()
                .map(ReportRow::toCsvLine)
                .collect(Collectors.joining("\n"))
                .getBytes(StandardCharsets.UTF_8);
    }
}
```

**export** metodunda kullanılan **final** anahtar kelimesinin amacı ve bu yapının bilinen adı için aşağıdaki ifadelerden hangisi doğrudur?

- A) **final**, metodun dönüş değerinin bir kez hesaplandıktan sonra değişmeyeceğini belirtir. **Singleton** kalıbı kullanılmıştır.
- B) **final**, **ReportExporter** sınıfından kalıtım yapılmasını engeller. **Factory Method** kalıbı kullanılmıştır.
- C) **final**, metodun sonucunu ilk çağrıdan sonra önbelleğe *(cache)* alır. **Decorator** kalıbı kullanılmıştır.
- D) **final**, alt sınıfların ortak akışı *(veriyi yükleme + filtreleme + çıktı üretme)* ezmesini *(override)* engeller. Değişen adım ise **protected abstract** metot ile alt sınıflara bırakılır. **Template Method** kalıbı kullanılmıştır.

## Soru 5

Bir bildirim servisinde **Object** türünden gelen mesaj nesnesinin tipine göre farklı işlem yapmak için aşağıdaki gibi bir söz dizimi *(syntax)* kullanılmaktadır.

```java
public String describe(Object message) {
    if (message instanceof EmailMessage email) {
        return "E-posta: " + email.subject();
    }
    return "Bilinmeyen mesaj";
}
```

Bu söz dizimi ile ilgili aşağıdaki ifadeler ortaya atılmıştır:

- **I.** Tip kontrolü ve tip dönüşümü *(cast)* tek adımda yapılır. Ayrıca `(EmailMessage) message` şeklinde bir dönüşüm yazmaya gerek kalmaz.
- **II.** **email** değişkeni yalnızca koşulun **true** olduğu kapsamda *(scope)* kullanılabilir.
- **III.** Bu söz dizimi **Java 8** sürümünden beri dilde yer almaktadır.
- **IV.** **message** değişkeni **null** ise bu ifade **NullPointerException** fırlatır.

Sizce hangi ifade/ifadeler doğrudur?

- A) I ve III
- B) Yalnız I
- C) I ve II
- D) Hepsi

## Soru 6

Token maliyeti hesaplayan bir hiyerarşide üst sınıf *(base class)* şu sözleşmeyi *(contract)* vaat etmektedir; *"Her zaman geçerli ve sıfırdan büyük veya eşit bir değer döner, asla exception fırlatmaz. Pozitif token tüketimi son geçerli birim fiyat üzerinden ücretlendirilir. Bağlantı kesildiğinde kullanılabilecek geçerli bir fiyat yerel olarak saklanır."* Bu taban sınıftan türeyen sınıflardan birisi de aşağıdaki gibidir.

```java
public class OfflineTokenCalculator extends TokenCalculator {
    private boolean isConnected = false;

    @Override
    public double calculateCost(long tokens) {
        if (!isConnected) {
            throw new UnsupportedOperationException(
                "Offline modu: Token hesaplama hizmeti kullanılamıyor.");
        }
        return 0;
    }
}
```

`TokenCalculator` türünden bir değişken bekleyen istemci kod, bu sınıfın bir örneği verildiğinde çalışma zamanında hata almaktadır. Sizce burada hangi ilke ihlal edilmiştir ve en uygun çözüm hangisidir?

- A) **Liskov Substitution Principle *(LSP)*** ihlal edilmiştir. Alt sınıf, temel sınıfın yerine kullanılamaMAktadır. Çözüm olarak bağlantı yokken de sözleşmeyi koruyan *(örneğin önbellekteki son bilinen oranla hesaplayan)* bir implementasyon yazılmalı, bağlantı yönetimi ise **Connectable** gibi ayrı bir rol arayüzüne alınmalıdır.
- B) **Interface Segregation Principle *(ISP)*** ihlal edilmiştir. Çözüm olarak **calculateCost** metodu iki farklı arayüze bölünmelidir.
- C) **Open/Closed Principle *(OCP)*** ihlal edilmiştir. Çözüm olarak temel sınıfa **if (calculator instanceof OfflineTokenCalculator)** kontrolü eklenmelidir.
- D) **Liskov Substitution Principle *(LSP)*** ihlal edilmiştir. Çözüm olarak exception fırlatmak yerine bağlantı yokken sessizce **0** döndürülmelidir.

## Soru 7

Aşağıdaki arayüzün sözleşmesi yorum satırlarında belirtilmiştir.

```java
/*
  Girdi     : Metot her long değerini kabul eder.
  Sonuç     : tokens <= 0 için 0 döner.
              Pozitif girdilerde sonuç deterministik ve negatif olmayan bir değerdir.
  Exception : Metot exception fırlatmaz.
  Alt tipler bu sözleşmeyi zayıflatamaz, sadece daha güçlü garantiler ekleyebilir.
*/
public interface TokenCalculator {
    double calculateCost(long tokens);
}
```

Aşağıdaki implementasyonlardan hangisi **Liskov Substitution Principle** açısından bu sözleşmeye uygundur? *(RATE = 0.002 kabul ediniz)*

A)

```java
public double calculateCost(long tokens) {
    return tokens * RATE;
}
```

B)

```java
public double calculateCost(long tokens) {
    if (tokens > 1_000_000) {
        throw new IllegalArgumentException("Çok büyük talep");
    }
    return tokens <= 0 ? 0 : tokens * RATE;
}
```

C)

```java
public double calculateCost(long tokens) {
    return tokens <= 0 ? 0 : tokens * RATE * (0.9 + Math.random() * 0.2);
}
```

D)

```java
public double calculateCost(long tokens) {
    double commission = 1.10; // %10 platform komisyonu
    return tokens <= 0 ? 0 : tokens * RATE * commission;
}
```

## Soru 8

Bellek üzerinde *(in-memory)* çalışan bir veri erişim katmanında tüm entity türleri için aşağıdaki generic arayüz tanımlanmıştır. **Customer** sınıfının **title** alanı vardır ancak **Invoice** sınıfının böyle bir alanı yoktur.

```java
public interface PersistenceService<T extends Entity> {
    void save(T entity);
    void delete(T entity);
    T findById(Long id);
    List<T> findByTitle(String title);
}

public class InvoicePersistenceService implements PersistenceService<Invoice> {
    // save, delete, findById implementasyonları...

    @Override
    public List<Invoice> findByTitle(String title) {
        // Invoice nesnelerinin title alanı yok!
        return null;
    }
}
```

Bu tasarımdaki problemi **Interface Segregation Principle *(ISP)*** doğrultusunda çözmek için aşağıdakilerden hangisi tercih edilmelidir?

- A) **Invoice** sınıfına kullanılmayacak olsa da boş bir **title** alanı eklemek.
- B) **findByTitle** metodunu `TitleSearchable<T extends Entity>` isimli ayrı bir rol arayüzüne *(role interface)* taşımak; **CustomerPersistenceService** sınıfının hem **PersistenceService** hem de **TitleSearchable** arayüzlerini, **InvoicePersistenceService** sınıfının ise yalnızca **PersistenceService** arayüzünü uygulamasını sağlamak.
- C) **null** döndürmek yerine **UnsupportedOperationException** fırlatmak.
- D) **findByTitle** metodunu arayüz içinde `default` metot olarak tanımlayıp varsayılan olarak **null** döndürmesini sağlamak.

## Soru 9

Kodları aşağıda verilen DataInspectorService sınıfı veriyi **JSON** formatına çevirip bir dosyaya yazmak üzere kodlanmış writeData isimli bir fonksiyonellik sağlamaktadır.

```java
public class DataInspectorService {
    public void writeData(Data data, String destination) throws IOException {
        JsonFormatter jsonFormatter = new JsonFormatter();
        try (PrintWriter writer = new PrintWriter(new FileWriter(destination))) {
            writer.write(jsonFormatter.format(data));
        }
    }
}
```

İlerleyen zamanlarda **XML** ve **CSV** formatlarının da desteklenmesi, ayrıca çıktının dosya yerine konsola ya da belleğe yazılabilmesi de istenmektedir. **Dependency Inversion Principle *(DIP)*** açısından bu vakayı değerlendirdiğimizde, aşağıdaki tasarımlardan hangisi en uygun çözüm olur?

A)

```java
public interface Formatter {
    String format(Data data);
}

public class DataInspectorService {
    private final Formatter formatter;

    public DataInspectorService(Formatter formatter) {
        this.formatter = formatter;
    }

    public void writeData(Data data, Writer writer) throws IOException {
        writer.write(formatter.format(data));
        writer.flush();
    }
}
```

B)

```java
public class DataInspectorService {
    private final JsonFormatter formatter = new JsonFormatter();

    public void writeData(Data data, String destination) throws IOException {
        try (PrintWriter writer = new PrintWriter(new FileWriter(destination))) {
            writer.write(formatter.format(data));
        }
    }
}
```

C)

```java
public class DataInspectorService {
    public void writeData(Data data, String destination, String format) throws IOException {
        String output = switch (format) {
            case "json" -> new JsonFormatter().format(data);
            case "xml"  -> new XmlFormatter().format(data);
            default     -> new CsvFormatter().format(data);
        };
        Files.writeString(Path.of(destination), output);
    }
}
```

D)

```java
public class DataInspectorService extends JsonFormatter {
    public void writeData(Data data, String destination) throws IOException {
        Files.writeString(Path.of(destination), format(data));
    }
}
```

## Soru 10

Bir uygulamada kullanıcılara gösterilecek karşılama mesajları aşağıdaki arayüz ile üretilmektedir. Arayüzde `@FunctionalInterface` notasyonu *(annotation)* **bulunmamaktadır**. **User** ise `record User(int id, String name)` şeklinde tanımlanmış bir **record** türüdür.

```java
public interface Greeter {
    String greet(User user);
}
```

**throws IOException** bildirimi olan bir test metodunda aşağıdaki kod parçası çalıştırılıyor olsun.

```java
Greeter greeter = u -> "Merhaba " + u.name();

StringWriter out = new StringWriter();
out.write(greeter.greet(new User(7, "Ada")));

System.out.println(out.toString().strip());
```

Sizce bu kodun çıktısı ne olur?

- A) Kod derlenmez zira **Greeter** arayüzünde `@FunctionalInterface` notasyonu olmadığından lambda ifadesi atanamaz.
- B) `Merhaba Ada`
- C) `Merhaba User[id=7, name=Ada]`
- D) Kod derlenmez, çünkü lambda ifadeleri yalnızca **java.util.function** paketindeki arayüzlere atanabilir.

## Soru 11

Bir CSV formatlayıcısı yazdığımızı düşünelim. Alanları bir separator ile birleştirmek için ilk etapta aşağıdaki gibi bir kod yazdığımızı varsayalım.

```java
int id = 42;
char separator = ',';
String line = id + separator + "hello";
System.out.println(line);
```

Sizce bu kodun çıktısı ne olur? *(İpucu: ',' karakterinin Unicode/ASCII değeri 44'tür.)*

- A) `42,hello`
- B) `42hello`
- C) Kod derlenmez, **int** ile **char** toplanamaz.
- D) `86hello`

## Soru 12

Bir e-ticaret sitesinde kargo ücreti teslimat tipine *(STANDARD, EXPRESS, PICKUP_POINT, SAME_DAY)* göre hesaplanmaktadır. İlk sürümde tüm kurallar tek bir metot içinde **switch** ve iç içe **if** blokları ile yazılmış, statik kod analizi aracı yüksek **Cognitive Complexity** uyarısı vermiştir. Ekip, her teslimat tipinin fiyatlandırma kuralını ayrı bir sınıfa taşıyarak aşağıdaki tasarıma karar vermiştir.

```text
┌─────────────────────────────────┐      use     ┌──────────────── [ ? ] ─────────────────┐
│ ShippingCostCalculator          │─────────────>│                                        │
│ - strategies:                   │              │                                        │
│   EnumMap<DeliveryType, ?>      │              │                                        │
│ + calculate(shipment, type)     │              └───────────────────▲────────────────────┘
└─────────────────────────────────┘                                  │ implements
                    ┌───────────────────┬─────────────────────┬──────┴──────────────┐
                    │                   │                     │                     │
          ┌─────────┴────────┐ ┌────────┴────────┐ ┌──────────┴──────────┐ ┌────────┴────────┐
          │ StandardShipping │ │ ExpressShipping │ │ PickupPointShipping │ │ SameDayShipping │
          └──────────────────┘ └─────────────────┘ └─────────────────────┘ └─────────────────┘
```

Diyagramda **[ ? ]** ile gösterilen eksik parça için aşağıdaki arayüzlerden hangisi, her stratejinin sorumluluğunu yalnızca seçilen teslimat tipinin ücretini hesaplamakla sınırlar?

A)

```java
public abstract class ShippingStrategy {
    public abstract double calculate(Shipment shipment, DeliveryType type);
    // her alt sınıf içinde type değerine göre switch yapılır
}
```

B)

```java
public enum ShippingStrategy {
    STANDARD, EXPRESS, PICKUP_POINT, SAME_DAY
}
```

C)

```java
@FunctionalInterface
public interface ShippingStrategy {
    double calculate(Shipment shipment);
}
```

D)

```java
public interface ShippingStrategy {
    double calculate(Shipment shipment);
    DeliveryType nextDeliveryType(); // bir sonraki teslimat tipine kendisi karar verir
}
```

## Soru 13

Bir e-ticaret sitesinde müşteri seviyesine *(BRONZE, SILVER, GOLD)* göre indirim uygulanmaktadır. İndirim kuralları **DiscountPolicy** arayüzünü uygulayan sınıflarda tutulmakta ve aşağıdaki **DiscountCalculator** sınıfı tarafından kullanılmaktadır.

```java
@FunctionalInterface
public interface DiscountPolicy {
    double discount(Order order);
}

public class DiscountCalculator {
    private final Map<CustomerTier, DiscountPolicy> policies = new EnumMap<>(CustomerTier.class);

    public DiscountCalculator() {
        policies.put(CustomerTier.BRONZE, new BronzeDiscount());
        policies.put(CustomerTier.SILVER, new SilverDiscount());
        policies.put(CustomerTier.GOLD, new GoldDiscount());
    }

    public void register(CustomerTier tier, DiscountPolicy policy) {
        policies.put(tier, policy);
    }

    public double calculate(Order order, CustomerTier tier) {
        DiscountPolicy policy = policies.get(tier);
        if (policy == null) {
            throw new IllegalArgumentException("Bilinmeyen müşteri seviyesi: " + tier);
        }
        return Math.round(policy.discount(order) * 100) / 100.0;
    }
}
```

Kampanya döneminde **GOLD** müşterilere indirim uygulanmaması istendiği için aşağıdaki satır çalıştırılmıştır.

```java
calculator.register(CustomerTier.GOLD, order -> 0.0);
```

Eklenen bu yeni kod parçası ile ilgili aşağıdaki ifadeler ortaya atılmıştır.

- **I.** **DiscountPolicy** tek bir soyut metoda sahip olduğundan, yeni bir sınıf yazmadan lambda ifadesi ile politika verilebilir.
- **II.** **EnumMap** yalnızca belirtilen enum türünün sabitlerini anahtar olarak kabul eden, enum için optimize edilmiş bir **Map** implementasyonudur.
- **III.** **register** çağrısından sonra **GOLD** seviyesi için **calculate** metodu **0.0** döner. Çünkü mevcut politika yenisi ile değiştirilmiştir.
- **IV.** İleride **PLATINUM** gibi yeni bir müşteri seviyesi eklemek için **calculate** metoduna yeni bir **case** eklenmesi zorunludur.

Sizce bu ifadelerden hangisi/hangileri doğrudur?

- A) I, II ve III
- B) Hepsi
- C) I ve IV
- D) Yalnız II

## Soru 14

Bir e-ticaret platformunda siparişler **NEW -> PAID -> SHIPPED -> DELIVERED** şeklinde ilerlemekte, belirli aşamalarda ise **CANCELLED** durumuna geçebilmektedir. Mevcut **Order** sınıfında **pay()**, **ship()**, **deliver()** ve **cancel()** metotlarının her biri `switch (status)` bloğu ile başlamakta ve aynı durum kontrolleri tüm metotlarda tekrar etmektedir. Ayrıca *"ödenmiş sipariş iptal edilirse iade başlatılır"*, *"kargodaki sipariş iptal edilemez"* gibi kurallar bu bloklara dağılmış durumdadır. Amaç her durumun kendi davranışını bilmesi ve bir sonraki geçişe **durumun kendisinin** karar vermesidir.

Sizce bu senaryo için en uygun tasarım kalıbı hangisidir?

- A) **Strategy** - Çünkü hangi algoritmanın çalışacağını istemci seçer ve stratejiler birbirinden habersizdir.
- B) **State** - Çünkü nesnenin davranışı içinde bulunduğu duruma göre değişmekte, her durum ayrı bir sınıf olarak modellenmekte ve geçişlere durum nesneleri karar vermektedir.
- C) **Command** - Çünkü her sipariş işlemi geri alınabilir *(undo)* bir komut olarak saklanmalıdır.
- D) **Adapter** - Çünkü sipariş durumları dış bir sistemin formatına dönüştürülmelidir.

## Soru 15

Bir destek talebi *(ticket)* sisteminde taleplerin yaşam döngüsü **OPEN -> ASSIGNED -> RESOLVED -> CLOSED** şeklindedir ve bazı aşamalarda talep **CANCELLED** durumuna geçebilmektedir. Her durum ayrı bir sınıf olarak aşağıdaki gibi modellenmiştir *(kodun sadece ilgili kısımları verilmiştir)*.

```java
public interface TicketState {
    String name();
    default void assign(Ticket ticket)  { throw invalid("atanamaz"); }
    default void resolve(Ticket ticket) { throw invalid("çözüldü olarak işaretlenemez"); }
    default void close(Ticket ticket)   { throw invalid("kapatılamaz"); }
    default void cancel(Ticket ticket)  { throw invalid("iptal edilemez"); }

    private IllegalStateException invalid(String action) {
        return new IllegalStateException(name() + " durumundaki talep " + action);
    }
}

final class AssignedState implements TicketState {
    public String name() { return "ASSIGNED"; }
    public void resolve(Ticket ticket) { ticket.transitionTo(new ResolvedState(), "Çözüldü"); }
    public void cancel(Ticket ticket)  { ticket.transitionTo(new CancelledState(), "İptal edildi, müşteriye bilgi verildi"); }
}

final class ResolvedState implements TicketState {
    public String name() { return "RESOLVED"; }
    public void close(Ticket ticket) { ticket.transitionTo(new ClosedState(), "Kapatıldı"); }
    public void cancel(Ticket ticket) {
        throw new IllegalStateException("Çözülmüş talep iptal edilemez, yeniden açma talebi oluşturulmalı");
    }
}
```

Aşağıdaki kod parçası çalıştırıldığında ekran çıktısı ne olur? *(OpenState sınıfının assign metodu talebi AssignedState durumuna geçirmektedir. `transitionTo` metoduna verilen geçiş mesajları talebin geçmişine kaydedilir; konsola yalnızca gösterilen `println` çağrıları yazar. `getStatus()` metodu aktif durumun adını döndürür.)*

```java
Ticket ticket = new Ticket("DT-1001");
ticket.assign();
ticket.resolve();
try {
    ticket.cancel();
} catch (IllegalStateException e) {
    System.out.println(e.getMessage());
}
System.out.println(ticket.getStatus());
```

- A)
  ```text
  İptal edildi, müşteriye bilgi verildi
  CANCELLED
  ```
- B) Kod derlenmez, çünkü **transitionTo** metodu **package-private** tanımlanmıştır.
- C)
  ```text
  Çözülmüş talep iptal edilemez, yeniden açma talebi oluşturulmalı
  RESOLVED
  ```
- D)
  ```text
  RESOLVED durumundaki talep iptal edilemez
  RESOLVED
  ```

## Soru 16

Bir bankanın havale/EFT modülünde transfer talebi sırasıyla şu kontrollerden geçmektedir; *IBAN formatı -> alıcının kara listede olup olmadığı -> bakiye -> günlük limit -> 50.000 birim üzeri işlemlerde açıklama zorunluluğu*. İlk sürümde bu kontroller dokuz seviye iç içe **if** bloğundan oluşan *"arrow code"* biçiminde yazılmıştır. Kontrollerden herhangi biri başarısız olduğunda akış kesilmeli, başarılıysa talep bir sonraki kontrole devredilmelidir. Bu nedenle kod tekrardan gözden geçirilmiş ve **Chain of Responsibility** kalıbı tercih edilerek ve aşağıdaki soyut sınıf *(abstract class)* yazılmıştır.

```java
public abstract class TransferCheck {
    private TransferCheck next;

    public TransferCheck linkWith(TransferCheck next) {
        this.next = next;
        return next;
    }

    public final TransferResult check(TransferRequest request) {
        Optional<String> rejection = verify(request);
        if (rejection.isPresent()) {
            return TransferResult.reject(rejection.get());
        }
        if (next == null) {
            return TransferResult.approve();
        }
        ...................................
    }

    protected abstract Optional<String> verify(TransferRequest request);
}
```

Sizce boş bırakılan satıra *(...)* aşağıdaki ifadelerden hangisi gelmelidir?

- A) `return next.verify(request);`
- B) `return TransferResult.approve();`
- C) `next = null; return check(request);`
- D) `return next.check(request);`

## Soru 17

Döviz kuru getiren bir servise önbellekleme *(cache)*, loglama ve geçici hatalarda yeniden deneme *(retry)* yetenekleri **Decorator** kalıbı ile eklenmiştir. Tüm sınıflar **ExchangeRateProvider** arayüzünü uygulamaktadır. Davranışlar şöyledir;

- **RemoteRateClient(2)**: Uzak servisi temsil eder, ilk **2** çağrıda zaman aşımı *(IllegalStateException)* fırlatır, sonrakilerde kur değerini döner. **getCallCount()** uzak servise kaç kez gidildiği bilgisini verir.
- **RetryingRateProvider(delegate, 3)**: **IllegalStateException** alındığında en fazla **3** deneme yapar.
- **CachingRateProvider(delegate)**: Sonuçları `computeIfAbsent` ile önbelleğe alır. Bir döviz kuru çifti için başarılı ve null olmayan bir sonuç önbellekte bulunduğu sürece sarmaladığı nesneye tekrar gitmez. Hesaplama exception ile biterse önbelleğe kayıt eklenmez.
- **LoggingRateProvider(delegate, log)**: Her çağrıda istek ve başarılı sonuç için birer log satırı yazar.

```java
RemoteRateClient client = new RemoteRateClient(2);
List<String> log = new ArrayList<>();

ExchangeRateProvider provider =
        new LoggingRateProvider(
                new CachingRateProvider(
                        new RetryingRateProvider(
                                new RemoteExchangeRateProvider(client), 3)),
                log::add);

provider.getRate("USD/TRY");
provider.getRate("USD/TRY");

System.out.println(client.getCallCount() + " / " + log.size());
```

Sizce bu kodun çalışma zamanı çıktısı ne olur?

- A) `3 / 4`
- B) `6 / 4`
- C) `3 / 2`
- D) `1 / 4`

## Soru 18

Bir e-ticaret uygulaması iki farklı sanal POS sağlayıcısı *(provider)* ile çalışmaktadır. Sağlayıcıların kütüphaneleri bize ait değildir ve değiştirilemez.

- **PayFastClient**: Tutarı kuruş cinsinden **long** olarak bekler *(149.90 TL → 14990)* ve sonucu sayısal bir kod ile döner *(0 = başarılı)*.
- **LegacyBankPos**: XML metni ile çalışır, tutarı metinsel bilgi olarak ve virgüllü bekler *("1250,50")*, cevap kodu **"00"** ise başarılıdır.

İş akışının *(CheckoutService)* bu detayları bilmeMEsi ve yalnızca kendi dilinde tasarlanmış bir arayüzle *(`BigDecimal` türünde, iki ondalıklı TL tutarı ve PaymentResult dönüşü)* konuşması hedeflenmektedir. Aşağıdaki çizelgede yer alan tasarım ile ilerlenmesi kararlaştırılmıştır.

```text
┌──────────────────┐      use     ┌────────────────────────────────────┐
│ CheckoutService  │─────────────>│ <<interface>> PaymentGateway       │
└──────────────────┘              │ + charge(orderNo,                  │
                                  │     BigDecimal amount)             │
                                  │   : PaymentResult                  │
                                  └─────────────────▲──────────────────┘
                                                    │ implements
                          ┌─────────────────────────┴──────────────────────┐
                          │                                                │
                 ┌────────┴─────────┐                          ┌───────────┴───────────┐
                 │      [ ? ]       │                          │ LegacyBankPosAdapter  │
                 └────────┬─────────┘                          └───────────┬───────────┘
                          │ has-a                                          │ has-a
                 ┌────────▼──────────────────┐                 ┌───────────▼───────────┐
                 │ PayFastClient             │                 │ LegacyBankPos         │
                 │ + makePayment(ref,        │                 │ + process(xml)        │
                 │   long kurus, currency)   │                 └───────────────────────┘
                 └───────────────────────────┘
```

Sizce **[ ? ]** ile gösterilen eksik parça için aşağıdakilerden hangisi kullanılmalıdır?

A)

```java
public class PayFastAdapter extends PayFastClient {
    // PaymentGateway uygulanmaz, CheckoutService doğrudan makePayment çağırır
}
```

B)

```java
public class PayFastAdapter implements PaymentGateway {
    private final PayFastClient client;

    public PayFastAdapter(PayFastClient client) { this.client = client; }

    @Override
    public PaymentResult charge(String orderNo, BigDecimal amount) {
        long amountInKurus = amount.movePointRight(2).longValueExact();
        PayFastResponse response = client.makePayment(orderNo, amountInKurus, "TRY");
        return response.code() == 0
                ? PaymentResult.ok(response.transactionRef())
                : PaymentResult.failed("Ödeme alınamadı (PayFast kodu: " + response.code() + ")");
    }
}
```

C)

```java
public class CheckoutService {
    public PaymentResult pay(String provider, String orderNo, BigDecimal amount) {
        if ("PAYFAST".equals(provider)) { /* kuruş dönüşümü + kod kontrolü */ }
        else if ("BANK".equals(provider)) { /* XML oluşturma + "00" kontrolü */ }
        return PaymentResult.failed("Bilinmeyen sağlayıcı");
    }
}
```

D)

```java
public interface PaymentGateway {
    // Hedef arayüz PayFast'e uyacak şekilde değiştirilir
    PayFastResponse charge(String orderNo, long amountInKurus, String currencyCode);
}
```

## Soru 19

Sıra tabanlı bir strateji oyununda hamlelerin geri alınması *(undo)* ve yinelenmesi *(redo)* için **Command** tasarım kalıbı kullanılmıştır. **CommandHistory** sınıfı aşağıdaki gibi tasarlanmıştır.

```java
public class CommandHistory {
    private final Deque<Command> undoStack = new ArrayDeque<>();
    private final Deque<Command> redoStack = new ArrayDeque<>();

    public void execute(Command command) {
        command.execute();
        undoStack.push(command);
        redoStack.clear();
    }

    public boolean undo() {
        if (undoStack.isEmpty()) return false;
        Command command = undoStack.pop();
        command.undo();
        redoStack.push(command);
        return true;
    }

    public boolean redo() {
        if (redoStack.isEmpty()) return false;
        Command command = redoStack.pop();
        command.execute();
        undoStack.push(command);
        return true;
    }
}
```

**MoveUnitCommand(board, unit, dx, dy)** birimi verilen miktar kadar hareket ettirir, **undo** çağrıldığında ise önceki konumuna geri götürür. Örnek bir kullanım senaryosu aşağıdaki gibidir.

```java
GameBoard board = new GameBoard();
board.addUnit("Okçu", new Position(0, 0), 100);
CommandHistory history = new CommandHistory();

history.execute(new MoveUnitCommand(board, "Okçu", 1, 0));
history.execute(new MoveUnitCommand(board, "Okçu", 0, 2));
history.undo();
history.execute(new MoveUnitCommand(board, "Okçu", 3, 0));
boolean result = history.redo();
```

Yukarıdaki kod çalıştırıldığında *Okçu* biriminin son konumu ve **result** değişkeninin değeri ne olur?

- A) Konum `(1, 2)`, result `true`
- B) Konum `(1, 2)`, result `false`
- C) Konum `(4, 2)`, result `true`
- D) Konum `(4, 0)`, result `false`

## Soru 20

Kod incelemelerinde *(code review)* karşılaşılan belirtilerden hangi tasarım kalıbına bakılması gerektiğini bilmek, doğru refactoring kararını vermeyi kolaylaştırır. Aşağıdaki tabloda kodda görülen belirtiler ve ilgili tasarım kalıpları yer almaktadır.

| | **Kodda görülen belirti** | | **Harf** | **Tasarım kalıbı** |
| --- | --- | --- | --- | --- |
| 1 | Bir **state alanına** göre dallanan ve birden çok metotta tekrar eden `switch` blokları | | a | **Chain of Responsibility** |
| 2 | Asıl işin etrafına eklenen `cache/log/retry` gibi yetenekler, metotlara eklenen `boolean` türü parametreler | | b | **Command** |
| 3 | Geri alma *(undo)*, yineleme *(redo)*, kuyruğa alma *(queue)* veya işlemleri tekrar oynatma *(replay)* ihtiyacı | | c | **Decorator** |
| 4 | Sırayla çalışan ve herhangi biri akışı kesebilen kontroller, iç içe `if` blokları | | d | **State** |

Sizce doğru eşleştirme hangisidir?

- A) 1-a, 2-c, 3-b, 4-d
- B) 1-d, 2-b, 3-c, 4-a
- C) 1-d, 2-c, 3-b, 4-a
- D) 1-c, 2-d, 3-a, 4-b

---

## Cevap Anahtarı

| Soru No | Doğru Cevap |
| ------- | ----------- |
| 1 | C |
| 2 | A |
| 3 | B |
| 4 | D |
| 5 | C |
| 6 | A |
| 7 | D |
| 8 | B |
| 9 | A |
| 10 | B |
| 11 | D |
| 12 | C |
| 13 | A |
| 14 | B |
| 15 | C |
| 16 | D |
| 17 | A |
| 18 | B |
| 19 | D |
| 20 | C |
