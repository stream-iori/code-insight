package com.example.repository;

import com.example.model.User;
import java.util.*;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicLong;
import java.util.stream.Collectors;

/**
 * Concrete implementation of UserRepository
 * Demonstrates repository pattern with in-memory storage
 */
public class UserRepositoryImpl implements UserRepository {
    private final Map<Long, User> users = new ConcurrentHashMap<>();
    private final AtomicLong idGenerator = new AtomicLong(1);
    
    @Override
    public Optional<User> findById(Long id) {
        return Optional.ofNullable(users.get(id));
    }
    
    @Override
    public List<User> findAll() {
        return new ArrayList<>(users.values());
    }
    
    @Override
    public List<User> findByUsernameContaining(String username) {
        return users.values().stream()
                .filter(user -> user.getUsername().toLowerCase()
                        .contains(username.toLowerCase()))
                .collect(Collectors.toList());
    }
    
    @Override
    public Optional<User> findByEmail(String email) {
        return users.values().stream()
                .filter(user -> user.getEmail().equalsIgnoreCase(email))
                .findFirst();
    }
    
    @Override
    public List<User> findByActiveTrue() {
        return users.values().stream()
                .filter(User::isActive)
                .collect(Collectors.toList());
    }
    
    @Override
    public List<User> findByRole(User.UserRole role) {
        return users.values().stream()
                .filter(user -> user.getRole() == role)
                .collect(Collectors.toList());
    }
    
    @Override
    public User save(User user) {
        if (user.getId() == null) {
            user.setId(idGenerator.getAndIncrement());
        }
        users.put(user.getId(), user);
        return user;
    }
    
    @Override
    public void deleteById(Long id) {
        users.remove(id);
    }
    
    @Override
    public boolean existsByEmail(String email) {
        return findByEmail(email).isPresent();
    }
    
    @Override
    public boolean existsByUsername(String username) {
        return users.values().stream()
                .anyMatch(user -> user.getUsername().equalsIgnoreCase(username));
    }
    
    @Override
    public long count() {
        return users.size();
    }
}