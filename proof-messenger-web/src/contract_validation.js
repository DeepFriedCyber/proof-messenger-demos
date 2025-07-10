/**
 * Interface Contract Validation for WASM Exports
 * 
 * This module provides comprehensive contract validation for all WASM exports,
 * ensuring type safety, parameter validation, and exception handling.
 */

import init, {
    WasmKeyPair,
    WasmMessage,
    generate_invite_code,
    validate_invite_code,
    bytes_to_hex,
    hex_to_bytes,
    verify_signature,
    validate_public_key,
    validate_signature,
    console_log,
    console_warn,
    console_error
} from '../pkg/proof_messenger_web.js';

/**
 * Contract validation errors
 */
class ContractViolationError extends Error {
    constructor(message, functionName, expectedContract, actualValue) {
        super(`Contract violation in ${functionName}: ${message}`);
        this.name = 'ContractViolationError';
        this.functionName = functionName;
        this.expectedContract = expectedContract;
        this.actualValue = actualValue;
    }
}

/**
 * Type validation utilities
 */
const TypeValidators = {
    isUint8Array: (value) => value instanceof Uint8Array,
    isString: (value) => typeof value === 'string',
    isBoolean: (value) => typeof value === 'boolean',
    isNumber: (value) => typeof value === 'number',
    isObject: (value) => typeof value === 'object' && value !== null,
    isFunction: (value) => typeof value === 'function',
    
    // Crypto-specific validators
    isValidPublicKey: (value) => value instanceof Uint8Array && value.length === 32,
    isValidPrivateKey: (value) => value instanceof Uint8Array && value.length === 32,
    isValidKeypair: (value) => value instanceof Uint8Array && value.length === 64,
    isValidSignature: (value) => value instanceof Uint8Array && value.length === 64,
    isValidInviteCode: (value) => typeof value === 'string' && value.length === 16 && /^[A-Z0-9]+$/.test(value),
    isValidHex: (value) => typeof value === 'string' && /^[0-9a-f]*$/i.test(value),
    isValidUUID: (value) => typeof value === 'string' && /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(value)
};

/**
 * Contract definitions for all WASM exports
 */
const Contracts = {
    // Function contracts
    generate_invite_code: {
        name: 'generate_invite_code',
        parameters: [],
        returns: { type: 'string', validator: TypeValidators.isValidInviteCode },
        throws: ['Error on generation failure'],
        description: 'Generates a cryptographically secure 16-character base32 invite code'
    },

    validate_invite_code: {
        name: 'validate_invite_code',
        parameters: [
            { name: 'code', type: 'string', required: true, validator: TypeValidators.isString }
        ],
        returns: { type: 'boolean', validator: TypeValidators.isBoolean },
        throws: [],
        description: 'Validates invite code format (16-character base32)'
    },

    bytes_to_hex: {
        name: 'bytes_to_hex',
        parameters: [
            { name: 'bytes', type: 'Uint8Array', required: true, validator: TypeValidators.isUint8Array }
        ],
        returns: { type: 'string', validator: TypeValidators.isValidHex },
        throws: ['TypeError for invalid input'],
        description: 'Converts bytes to hexadecimal string'
    },

    hex_to_bytes: {
        name: 'hex_to_bytes',
        parameters: [
            { name: 'hex', type: 'string', required: true, validator: (v) => TypeValidators.isString(v) && TypeValidators.isValidHex(v) }
        ],
        returns: { type: 'Uint8Array', validator: TypeValidators.isUint8Array },
        throws: ['Error for invalid hex string'],
        description: 'Converts hexadecimal string to bytes'
    },

    verify_signature: {
        name: 'verify_signature',
        parameters: [
            { name: 'pubkey_bytes', type: 'Uint8Array', required: true, validator: TypeValidators.isValidPublicKey },
            { name: 'data', type: 'Uint8Array', required: true, validator: TypeValidators.isUint8Array },
            { name: 'signature', type: 'Uint8Array', required: true, validator: TypeValidators.isValidSignature }
        ],
        returns: { type: 'boolean', validator: TypeValidators.isBoolean },
        throws: ['Error for invalid parameters'],
        description: 'Verifies a signature with separate public key'
    },

    validate_public_key: {
        name: 'validate_public_key',
        parameters: [
            { name: 'key_bytes', type: 'Uint8Array', required: true, validator: TypeValidators.isUint8Array }
        ],
        returns: { type: 'boolean', validator: TypeValidators.isBoolean },
        throws: [],
        description: 'Validates public key format'
    },

    validate_signature: {
        name: 'validate_signature',
        parameters: [
            { name: 'signature_bytes', type: 'Uint8Array', required: true, validator: TypeValidators.isUint8Array }
        ],
        returns: { type: 'boolean', validator: TypeValidators.isBoolean },
        throws: [],
        description: 'Validates signature format'
    },

    // Class contracts
    WasmKeyPair: {
        name: 'WasmKeyPair',
        constructor: {
            parameters: [],
            throws: ['Error on generation failure']
        },
        methods: {
            public_key_bytes: {
                parameters: [],
                returns: { type: 'Uint8Array', validator: TypeValidators.isValidPublicKey },
                throws: []
            },
            private_key_bytes: {
                parameters: [],
                returns: { type: 'Uint8Array', validator: TypeValidators.isValidPrivateKey },
                throws: []
            },
            keypair_bytes: {
                parameters: [],
                returns: { type: 'Uint8Array', validator: TypeValidators.isValidKeypair },
                throws: []
            },
            public_key_hex: {
                parameters: [],
                returns: { type: 'string', validator: (v) => TypeValidators.isString(v) && v.length === 64 },
                throws: []
            },
            sign: {
                parameters: [
                    { name: 'data', type: 'Uint8Array', required: true, validator: TypeValidators.isUint8Array }
                ],
                returns: { type: 'Uint8Array', validator: TypeValidators.isValidSignature },
                throws: ['Error for invalid data']
            }
        },
        static_methods: {
            from_bytes: {
                parameters: [
                    { name: 'bytes', type: 'Uint8Array', required: true, validator: TypeValidators.isValidKeypair }
                ],
                returns: { type: 'WasmKeyPair', validator: (v) => v instanceof WasmKeyPair },
                throws: ['Error for invalid keypair bytes']
            }
        }
    },

    WasmMessage: {
        name: 'WasmMessage',
        constructor: {
            parameters: [
                { name: 'sender', type: 'Uint8Array', required: true, validator: TypeValidators.isValidPublicKey },
                { name: 'recipient', type: 'Uint8Array', required: true, validator: TypeValidators.isValidPublicKey },
                { name: 'content', type: 'string', required: true, validator: TypeValidators.isString }
            ],
            throws: ['Error for invalid parameters']
        },
        methods: {
            sign: {
                parameters: [
                    { name: 'keypair_bytes', type: 'Uint8Array', required: true, validator: TypeValidators.isValidKeypair }
                ],
                returns: { type: 'void' },
                throws: ['Error for invalid keypair or signing failure']
            },
            verify: {
                parameters: [
                    { name: 'pubkey_bytes', type: 'Uint8Array', required: true, validator: TypeValidators.isValidPublicKey }
                ],
                returns: { type: 'boolean', validator: TypeValidators.isBoolean },
                throws: ['Error for invalid public key']
            },
            to_json: {
                parameters: [],
                returns: { type: 'string', validator: TypeValidators.isString },
                throws: ['Error on serialization failure']
            }
        },
        static_methods: {
            from_json: {
                parameters: [
                    { name: 'json', type: 'string', required: true, validator: TypeValidators.isString }
                ],
                returns: { type: 'WasmMessage|undefined', validator: (v) => v === undefined || v instanceof WasmMessage },
                throws: []
            }
        },
        properties: {
            id: { type: 'string', validator: TypeValidators.isValidUUID },
            sender_hex: { type: 'string', validator: (v) => TypeValidators.isString(v) && v.length === 64 },
            recipient_hex: { type: 'string', validator: (v) => TypeValidators.isString(v) && v.length === 64 },
            sender_bytes: { type: 'Uint8Array', validator: TypeValidators.isValidPublicKey },
            recipient_bytes: { type: 'Uint8Array', validator: TypeValidators.isValidPublicKey },
            content: { type: 'string', validator: TypeValidators.isString },
            timestamp: { type: 'string', validator: TypeValidators.isString },
            is_signed: { type: 'boolean', validator: TypeValidators.isBoolean }
        }
    }
};

/**
 * Contract validator class
 */
class ContractValidator {
    constructor() {
        this.violations = [];
        this.testResults = new Map();
    }

    /**
     * Validate function parameters
     */
    validateParameters(functionName, parameters, args) {
        const violations = [];

        // Check parameter count
        const requiredParams = parameters.filter(p => p.required);
        if (args.length < requiredParams.length) {
            violations.push(`Expected at least ${requiredParams.length} parameters, got ${args.length}`);
        }

        // Validate each parameter
        parameters.forEach((param, index) => {
            if (index < args.length) {
                const arg = args[index];
                if (!param.validator(arg)) {
                    violations.push(`Parameter '${param.name}' failed validation: expected ${param.type}, got ${typeof arg}`);
                }
            } else if (param.required) {
                violations.push(`Required parameter '${param.name}' is missing`);
            }
        });

        if (violations.length > 0) {
            throw new ContractViolationError(
                violations.join('; '),
                functionName,
                parameters,
                args
            );
        }
    }

    /**
     * Validate return value
     */
    validateReturn(functionName, returnSpec, actualValue) {
        if (returnSpec && !returnSpec.validator(actualValue)) {
            throw new ContractViolationError(
                `Return value failed validation: expected ${returnSpec.type}, got ${typeof actualValue}`,
                functionName,
                returnSpec,
                actualValue
            );
        }
    }

    /**
     * Create a contract-validated wrapper for a function
     */
    wrapFunction(originalFunction, contract) {
        const validator = this;
        
        return function contractValidatedWrapper(...args) {
            const functionName = contract.name;
            
            try {
                // Pre-condition: validate parameters
                validator.validateParameters(functionName, contract.parameters, args);
                
                // Execute original function
                const result = originalFunction.apply(this, args);
                
                // Post-condition: validate return value
                if (contract.returns) {
                    validator.validateReturn(functionName, contract.returns, result);
                }
                
                // Log successful execution
                validator.testResults.set(functionName, {
                    status: 'pass',
                    timestamp: new Date().toISOString(),
                    parameters: args.length,
                    returnType: typeof result
                });
                
                return result;
                
            } catch (error) {
                // Log contract violation or unexpected error
                const violation = {
                    functionName,
                    error: error.message,
                    timestamp: new Date().toISOString(),
                    parameters: args,
                    type: error instanceof ContractViolationError ? 'contract_violation' : 'unexpected_error'
                };
                
                validator.violations.push(violation);
                validator.testResults.set(functionName, {
                    status: 'fail',
                    error: error.message,
                    timestamp: new Date().toISOString()
                });
                
                // Re-throw with additional context
                if (error instanceof ContractViolationError) {
                    throw error;
                } else {
                    throw new Error(`Unexpected error in ${functionName}: ${error.message}`);
                }
            }
        };
    }

    /**
     * Create a contract-validated wrapper for a class
     */
    wrapClass(OriginalClass, contract) {
        const validator = this;
        
        // Create wrapper constructor
        function ContractValidatedClass(...args) {
            try {
                // Validate constructor parameters
                if (contract.constructor) {
                    validator.validateParameters(contract.name, contract.constructor.parameters, args);
                }
                
                // Create instance
                const instance = new OriginalClass(...args);
                
                // Wrap instance methods
                if (contract.methods) {
                    Object.keys(contract.methods).forEach(methodName => {
                        const methodContract = contract.methods[methodName];
                        const originalMethod = instance[methodName];
                        
                        if (typeof originalMethod === 'function') {
                            instance[methodName] = validator.wrapFunction(originalMethod.bind(instance), {
                                name: `${contract.name}.${methodName}`,
                                parameters: methodContract.parameters || [],
                                returns: methodContract.returns,
                                throws: methodContract.throws
                            });
                        }
                    });
                }
                
                // Validate properties
                if (contract.properties) {
                    Object.keys(contract.properties).forEach(propName => {
                        const propContract = contract.properties[propName];
                        const propValue = instance[propName];
                        
                        if (propValue !== undefined && !propContract.validator(propValue)) {
                            validator.violations.push({
                                functionName: `${contract.name}.${propName}`,
                                error: `Property validation failed: expected ${propContract.type}, got ${typeof propValue}`,
                                timestamp: new Date().toISOString(),
                                type: 'property_violation'
                            });
                        }
                    });
                }
                
                return instance;
                
            } catch (error) {
                validator.violations.push({
                    functionName: `${contract.name}.constructor`,
                    error: error.message,
                    timestamp: new Date().toISOString(),
                    parameters: args,
                    type: error instanceof ContractViolationError ? 'contract_violation' : 'unexpected_error'
                });
                
                throw error;
            }
        }
        
        // Copy static methods
        if (contract.static_methods) {
            Object.keys(contract.static_methods).forEach(methodName => {
                const methodContract = contract.static_methods[methodName];
                const originalMethod = OriginalClass[methodName];
                
                if (typeof originalMethod === 'function') {
                    ContractValidatedClass[methodName] = validator.wrapFunction(originalMethod, {
                        name: `${contract.name}.${methodName}`,
                        parameters: methodContract.parameters || [],
                        returns: methodContract.returns,
                        throws: methodContract.throws
                    });
                }
            });
        }
        
        // Copy prototype
        ContractValidatedClass.prototype = OriginalClass.prototype;
        ContractValidatedClass.prototype.constructor = ContractValidatedClass;
        
        return ContractValidatedClass;
    }

    /**
     * Get validation report
     */
    getReport() {
        return {
            totalTests: this.testResults.size,
            violations: this.violations.length,
            passedTests: Array.from(this.testResults.values()).filter(r => r.status === 'pass').length,
            failedTests: Array.from(this.testResults.values()).filter(r => r.status === 'fail').length,
            violations: this.violations,
            testResults: Object.fromEntries(this.testResults)
        };
    }

    /**
     * Clear validation history
     */
    reset() {
        this.violations = [];
        this.testResults.clear();
    }
}

/**
 * Create contract-validated WASM exports
 */
export async function createValidatedWasmExports() {
    // Initialize WASM
    await init();
    
    const validator = new ContractValidator();
    
    // Import original exports
    const {
        WasmKeyPair: OriginalWasmKeyPair,
        WasmMessage: OriginalWasmMessage,
        generate_invite_code: originalGenerateInviteCode,
        validate_invite_code: originalValidateInviteCode,
        bytes_to_hex: originalBytesToHex,
        hex_to_bytes: originalHexToBytes,
        verify_signature: originalVerifySignature,
        validate_public_key: originalValidatePublicKey,
        validate_signature: originalValidateSignature
    } = await import('../pkg/proof_messenger_web.js');
    
    // Create validated exports
    const validatedExports = {
        // Validated functions
        generate_invite_code: validator.wrapFunction(originalGenerateInviteCode, Contracts.generate_invite_code),
        validate_invite_code: validator.wrapFunction(originalValidateInviteCode, Contracts.validate_invite_code),
        bytes_to_hex: validator.wrapFunction(originalBytesToHex, Contracts.bytes_to_hex),
        hex_to_bytes: validator.wrapFunction(originalHexToBytes, Contracts.hex_to_bytes),
        verify_signature: validator.wrapFunction(originalVerifySignature, Contracts.verify_signature),
        validate_public_key: validator.wrapFunction(originalValidatePublicKey, Contracts.validate_public_key),
        validate_signature: validator.wrapFunction(originalValidateSignature, Contracts.validate_signature),
        
        // Validated classes
        WasmKeyPair: validator.wrapClass(OriginalWasmKeyPair, Contracts.WasmKeyPair),
        WasmMessage: validator.wrapClass(OriginalWasmMessage, Contracts.WasmMessage),
        
        // Validator utilities
        validator,
        ContractViolationError,
        TypeValidators,
        Contracts
    };
    
    return validatedExports;
}

export {
    ContractValidator,
    ContractViolationError,
    TypeValidators,
    Contracts
};