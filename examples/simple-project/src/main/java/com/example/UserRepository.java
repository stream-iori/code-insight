package com.example.repository;

import com.example.model.User;
import java.util.List;
import java.util.Optional;

/**
 * Interface for database operations with Users
 * This shows what an interface looks like in Java
 * Includes modern Java features like Optional and pagination
 */
public interface UserRepository {
    Optional<User> findById(Long id);
    List<User> findAll();
    List<User> findByUsernameContaining(String username);
    Optional<User> findByEmail(String email);
    List<User> findByActiveTrue();
    List<User> findByRole(User.UserRole role);
    User save(User user);
    void deleteById(Long id);
    boolean existsByEmail(String email);
    boolean existsByUsername(String username);
    long count();
}