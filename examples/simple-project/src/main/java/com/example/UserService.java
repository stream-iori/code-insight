package com.example.service;

import com.example.model.User;
import com.example.repository.UserRepository;
import org.springframework.stereotype.Service;
import java.util.List;

/**
 * Service layer - contains business logic
 * This shows a Spring @Service class
 */
@Service  // This tells Spring this is a service class
public class UserService {
    
    private final UserRepository userRepository;
    
    // Constructor injection - Spring provides the repository
    public UserService(UserRepository userRepository) {
        this.userRepository = userRepository;
    }
    
    public User getUserById(Long id) {
        return userRepository.findById(id);
    }
    
    public List<User> getAllUsers() {
        return userRepository.findAll();
    }
    
    public User createUser(User user) {
        return userRepository.save(user);
    }
    
    public void deleteUser(Long id) {
        userRepository.deleteById(id);
    }
}