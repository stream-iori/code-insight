/**
 * Copyright (c) 2024 Advanced Java Solutions Inc.
 * Comprehensive test file for Java structure parser with complex syntax features
 */

@FileLevelAnnotation(value = "complex-test-file", version = "1.0.0")
@AnotherAnnotation({"feature1", "feature2", "feature3"})
package com.example.advanced.java.parser.test;

import java.util.*;
import java.util.concurrent.ConcurrentHashMap;
import java.util.stream.Collectors;
import javax.validation.constraints.*;
import org.springframework.stereotype.*;
import org.springframework.web.bind.annotation.*;
import static java.lang.System.out;
import static java.util.Collections.*;

/**
 * Main test class demonstrating comprehensive Java syntax features
 * @author Parser Test Team
 * @version 2.1.0
 */
@Controller
@RequestMapping(path = "/api/v2", method = {RequestMethod.GET, RequestMethod.POST})
@Deprecated(since = "2.0.0", forRemoval = true)
public abstract class AdvancedTestClass<T extends Comparable<T> & Serializable, K, V>
        extends AbstractTestParent<T>
        implements TestInterface<K, V>, Serializable, Cloneable, AutoCloseable {
    
    // Complex field declarations
    @Autowired(required = false)
    @Qualifier("primaryService")
    private volatile UserService<T> userService;
    
    @Value("${app.config.maxRetries:3}")
    protected final int maxRetries = 5;
    
    @NotNull
    @Size(min = 1, max = 100)
    public static final String DEFAULT_NAME = "AdvancedTest";
    
    @Getter
    @Setter
    private transient Map<K, List<T>> complexMap = new ConcurrentHashMap<>();
    
    @Nullable
    private volatile T[] genericArray;
    
    // Nested enum with complex syntax
    public enum Status {
        ACTIVE("active", 1, new String[]{"on", "enabled"}),
        INACTIVE("inactive", 0, new String[]{"off", "disabled"}),
        PENDING("pending", 2, new String[]{"waiting", "queued"});
        
        private final String displayName;
        private final int code;
        private final String[] aliases;
        
        Status(String displayName, int code, String[] aliases) {
            this.displayName = displayName;
            this.code = code;
            this.aliases = aliases;
        }
        
        public String getDisplayName() { return displayName; }
        public int getCode() { return code; }
        public String[] getAliases() { return aliases; }
    }
    
    // Complex nested class with generics
    @Component
    protected static class InnerProcessor<U extends Number> {
        @Inject
        private Validator validator;
        
        @PostConstruct
        public void initialize() {
            out.println("InnerProcessor initialized");
        }
        
        @PreDestroy
        public void cleanup() {
            out.println("InnerProcessor cleanup");
        }
        
        @Transactional
        public List<U> processItems(@Valid List<U> items) {
            return items.stream()
                .filter(item -> item.doubleValue() > 0)
                .collect(Collectors.toList());
        }
    }
    
    // Nested interface
    @FunctionalInterface
    public interface DataProcessor<I, O> {
        O process(I input) throws ProcessingException;
        
        default <R> DataProcessor<I, R> andThen(DataProcessor<O, R> after) {
            return input -> after.process(process(input));
        }
    }
    
    // Nested record
    @ApplicationScoped
    public record ConfigRecord(
        @NotEmpty String name,
        @Min(1) int timeout,
        @Pattern(regexp = "^[a-z-]+$") String pattern,
        List<String> features
    ) implements Serializable {
        
        public ConfigRecord {
            Objects.requireNonNull(name, "name cannot be null");
            Objects.requireNonNull(features, "features cannot be null");
            features = unmodifiableList(new ArrayList<>(features));
        }
    }
    
    // Nested annotation
    @Target({ElementType.METHOD, ElementType.TYPE})
    @Retention(RetentionPolicy.RUNTIME)
    @Documented
    @interface ComplexAnnotation {
        String value() default "";
        int[] numbers() default {};
        Class<?>[] types() default {};
        Status status() default Status.ACTIVE;
        String[] aliases() default {};
        long timeout() default 5000L;
    }
    
    // Complex constructor
    @Autowired
    public AdvancedTestClass(@Qualifier("testUserService") UserService<T> userService,
                            @Value("${app.name}") String appName) {
        super(appName);
        this.userService = userService;
        out.println("AdvancedTestClass initialized with app: " + appName);
    }
    
    // Complex method with all possible features
    @GetMapping("/users/{userId}/profile")
    @ResponseBody
    @Cacheable(cacheNames = "userProfiles")
    @ComplexAnnotation(
        value = "get-user-profile",
        numbers = {1, 2, 3, 5, 8},
        types = {User.class, Profile.class},
        status = Status.ACTIVE,
        aliases = {"profile", "userProfile"},
        timeout = 30000L
    )
    public ResponseEntity<ProfileDto<T>> getUserProfile(
            @PathVariable("userId") @Valid @NotNull Long userId,
            @RequestParam(value = "includeDetails", defaultValue = "true") boolean includeDetails,
            @RequestHeader(value = "X-API-Key", required = false) String apiKey,
            @CookieValue(value = "session", required = false) String session,
            HttpServletRequest request) 
            throws UserNotFoundException, AccessDeniedException, SQLException {
        
        return ResponseEntity.ok()
            .header("X-Cache-Hit", "false")
            .body(userService.getUserProfile(userId, includeDetails));
    }
    
    // Generic method with type bounds
    @PostMapping("/process")
    @Transactional(rollbackFor = {Exception.class, RuntimeException.class})
    @Retryable(value = {ConnectException.class}, maxAttempts = 3)
    public <S extends T & Comparable<S>> List<S> processGenericData(
            @RequestBody @Valid List<S> data,
            @RequestParam(defaultValue = "10") int limit,
            @AuthenticationPrincipal User user) {
        
        return data.stream()
            .sorted()
            .limit(limit)
            .collect(Collectors.toList());
    }
    
    // Method with varargs and complex exception handling
    @PutMapping("/batch-update")
    @Async
    @EventListener(condition = "#event.type == 'bulk-update'")
    public CompletableFuture<Map<K, List<T>>> batchUpdateItems(
            @RequestParam("ids") Long... ids) 
            throws IllegalArgumentException, 
                   IllegalStateException, 
                   ConcurrentModificationException {
        
        return CompletableFuture.supplyAsync(() -> {
            Map<K, List<T>> result = new ConcurrentHashMap<>();
            Arrays.stream(ids)
                .map(userService::findById)
                .forEach(item -> result.put((K) item.getId(), Arrays.asList(item)));
            return result;
        });
    }
    
    // Abstract method implementation
    @Override
    public abstract T processData(K key, V value) throws ProcessingException;
    
    // Default method with lambda expressions
    @Override
    public void close() throws Exception {
        Optional.ofNullable(userService).ifPresent(service -> {
            try {
                service.cleanup();
            } catch (Exception e) {
                Thread.currentThread().interrupt();
                throw new RuntimeException("Cleanup failed", e);
            }
        });
    }
    
    // Static method with complex generics
    public static <X, Y> Map<X, List<Y>> groupByComplexKey(
            Collection<Y> items, 
            Function<Y, X> keyExtractor,
            Predicate<Y> filter) {
        
        return items.stream()
            .filter(filter)
            .collect(Collectors.groupingBy(keyExtractor, Collectors.toList()));
    }
    
    // Private method with complex exception handling
    @SneakyThrows
    private Optional<T> findItemByKey(K key) {
        try {
            return Optional.ofNullable(complexMap.get(key))
                .filter(list -> !list.isEmpty())
                .map(list -> list.get(0));
        } catch (NullPointerException | IndexOutOfBoundsException e) {
            log.warn("Failed to find item for key: {}", key, e);
            return Optional.empty();
        }
    }
    
    // Synchronized method with timeout
    @Timed(value = "test.method", description = "Time taken to execute synchronized method")
    public synchronized List<T> getCachedItems(long timeout, TimeUnit unit) 
            throws TimeoutException, InterruptedException {
        
        long startTime = System.nanoTime();
        long timeoutNanos = unit.toNanos(timeout);
        
        while (System.nanoTime() - startTime < timeoutNanos) {
            if (!complexMap.isEmpty()) {
                return complexMap.values().stream()
                    .flatMap(List::stream)
                    .collect(Collectors.toList());
            }
            Thread.sleep(100);
        }
        
        throw new TimeoutException("Timeout waiting for cached items");
    }
    
    // Complex field with initializer
    private final DataProcessor<T, K> dataProcessor = new DataProcessor<T, K>() {
        @Override
        public K process(T input) throws ProcessingException {
            return (K) input.toString();
        }
    };
    
    // Static nested class
    public static class Builder<B extends Builder<B>> {
        private String name;
        private int timeout = 5000;
        private List<String> features = new ArrayList<>();
        
        public B name(String name) {
            this.name = name;
            return (B) this;
        }
        
        public B timeout(int timeout) {
            this.timeout = timeout;
            return (B) this;
        }
        
        public B addFeature(String feature) {
            this.features.add(feature);
            return (B) this;
        }
        
        public AdvancedTestClass<String, Integer, Object> build() {
            return new AdvancedTestClass<String, Integer, Object>(null, name) {
                @Override
                public String processData(Integer key, Object value) {
                    return key + ":" + value;
                }
            };
        }
    }
    
    // Nested enum with annotation
    @Target(ElementType.FIELD)
    @Retention(RetentionPolicy.RUNTIME)
    public @interface FieldConfig {
        String description() default "";
        boolean required() default false;
        int order() default 0;
        String[] groups() default {};
    }
    
    // Final field with complex generic
    @FieldConfig(description = "Main processor", required = true, order = 1, groups = {"main", "core"})
    private final Supplier<CompletableFuture<Map<K, T>>> processor = () -> 
        CompletableFuture.supplyAsync(() -> {
            Map<K, T> map = new LinkedHashMap<>();
            complexMap.forEach((k, v) -> map.put(k, v.get(0)));
            return map;
        });
    
    // Native method declaration
    public native void nativeMethod(String input);
    
    // Strictfp method
    public strictfp double calculatePrecision(double... values) {
        double sum = 0.0;
        for (double value : values) {
            sum += value;
        }
        return sum;
    }
}